use std::{
    collections::{BTreeSet, HashSet},
    fs, io,
    path::{Path, PathBuf},
    thread,
};

use crate::{
    core::api_error::with_global_api_error_layer,
    plugin::contract::{
        AdminCliContribution, AdminMenuTree, AdminResourceContract, BackendApiContribution,
        CatalogItemContribution, CatalogItemKind, CatalogSource, ClientBootstrapPayload,
        ClientPageContribution, ClientPluginRecord, ContributionSet, DynAdminPluginProvider,
        GeneratedFileContribution, NativePluginContext, NativeRenderFn, NativeUiRenderer,
        NavItemContribution, PageContribution, PluginActivation, PluginDescriptor, PluginKind,
        PluginState, SettingsSectionContribution, ShellEntryContribution,
        ToolbarActionContribution, UiContribution, merge_menu_tree,
    },
};
use serde::{Deserialize, Serialize};

const PLUGIN_STATE_FILE: &str = "plugin-state.json";

pub fn load_native_snapshot(context: NativePluginContext, di: &mut rudi::Context) -> HostSnapshot {
    let enablement = load_plugin_enablement();
    NativePluginHost::from_context(context, di).load_snapshot_with_enablement(&enablement)
}

pub fn native_renderer(snapshot: &HostSnapshot, renderer_id: &str) -> Option<NativeRenderFn> {
    snapshot
        .native_renderers
        .iter()
        .find(|renderer| renderer.renderer_id == renderer_id)
        .map(|renderer| renderer.render)
}

pub async fn start_native_loopback_server(snapshot: HostSnapshot) -> anyhow::Result<String> {
    let app = snapshot.native_router.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let local_addr = listener.local_addr()?;
    thread::Builder::new()
        .name("aio-native-plugin-server".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("aio native plugin runtime failed: {error}");
                    return;
                }
            };
            runtime.block_on(async move {
                match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => {
                        if let Err(error) = axum::serve(listener, app).await {
                            eprintln!("aio native plugin server failed: {error}");
                        }
                    }
                    Err(error) => {
                        eprintln!("aio native plugin listener failed: {error}");
                    }
                }
            });
        })?;
    Ok(format!("http://{local_addr}"))
}

pub struct NativePluginHost {
    plugins: Vec<DynAdminPluginProvider>,
    context: NativePluginContext,
}

impl NativePluginHost {
    pub fn new(context: NativePluginContext) -> Self {
        Self {
            plugins: Vec::new(),
            context,
        }
    }

    pub fn from_context(mut context: NativePluginContext, di: &mut rudi::Context) -> Self {
        if context.shared_db.is_none() {
            context.shared_db = di.resolve_option::<crate::core::db::Db>();
        }
        let mut plugins = di.resolve_by_type::<DynAdminPluginProvider>();
        plugins.sort_by(|left, right| left.admin_descriptor().id.cmp(&right.admin_descriptor().id));
        Self { plugins, context }
    }

    pub fn with_plugin(mut self, plugin: DynAdminPluginProvider) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn load_snapshot(self) -> HostSnapshot {
        self.load_snapshot_with_enablement(&PluginEnablementStore::default())
    }

    pub fn load_snapshot_with_enablement(self, enablement: &PluginEnablementStore) -> HostSnapshot {
        let mut snapshot = HostSnapshot::default();
        let mut seen_ids = HashSet::new();
        let mut seen_routes = HashSet::new();

        for plugin in self.plugins {
            let descriptor = plugin.admin_descriptor();
            if !seen_ids.insert(descriptor.id.clone()) {
                snapshot.plugins.push(failed_record(
                    descriptor.clone(),
                    format!("duplicate plugin ID: {}", descriptor.id),
                ));
                continue;
            }
            if !enablement.plugin_enabled(&descriptor.id) {
                snapshot.plugins.push(disabled_record(descriptor));
                continue;
            }

            let admin_contribution = match plugin.admin_contribution() {
                Ok(c) => c,
                Err(error) => {
                    snapshot.plugins.push(failed_record(
                        descriptor.clone(),
                        format!(
                            "plugin `{}` failed during contributions: {}",
                            descriptor.id, error
                        ),
                    ));
                    continue;
                }
            };
            let contributions = admin_contribution.native.clone();

            let runtime = match plugin.admin_runtime(self.context.clone()) {
                Ok(r) => r,
                Err(error) => {
                    merge_snapshot_admin_contribution(&mut snapshot, admin_contribution);
                    merge_snapshot_contributions(&mut snapshot, contributions.clone());
                    snapshot
                        .plugin_contributions
                        .push(PluginContributionRecord {
                            plugin_id: descriptor.id.clone(),
                            contributions: contributions.clone(),
                        });
                    snapshot.plugins.push(failed_record(
                        descriptor.clone(),
                        format!(
                            "plugin `{}` failed during runtime: {}",
                            descriptor.id, error
                        ),
                    ));
                    continue;
                }
            };

            if let Some(startup) = runtime.startup
                && let Err(error) = startup(self.context.clone())
            {
                snapshot.plugins.push(failed_record(
                    descriptor.clone(),
                    format!(
                        "plugin `{}` failed during startup: {}",
                        descriptor.id, error
                    ),
                ));
                continue;
            }

            if let Some((method, path)) =
                first_duplicate_backend_route(&contributions.backend_apis, &mut seen_routes)
            {
                snapshot.plugins.push(failed_record(
                    descriptor.clone(),
                    format!("backend route duplicated: {method} {path}"),
                ));
                continue;
            }

            merge_snapshot_admin_contribution(&mut snapshot, admin_contribution);
            snapshot
                .plugin_contributions
                .push(PluginContributionRecord {
                    plugin_id: descriptor.id.clone(),
                    contributions: contributions.clone(),
                });
            merge_snapshot_contributions(&mut snapshot, contributions);
            snapshot.native_renderers.extend(runtime.renderers);
            snapshot.native_router = snapshot.native_router.merge(runtime.router);
            snapshot.plugins.push(PluginRuntimeRecord {
                descriptor,
                state: PluginState::Active,
                error: None,
            });
        }

        sort_snapshot(&mut snapshot);
        snapshot.native_router = with_global_api_error_layer(snapshot.native_router);
        snapshot
    }
}

pub fn load_plugin_enablement() -> PluginEnablementStore {
    read_plugin_enablement_store(&plugin_enablement_store_path()).unwrap_or_default()
}

pub fn set_plugin_enabled(plugin_id: &str, enabled: bool) -> io::Result<()> {
    set_plugin_enabled_at(plugin_enablement_store_path(), plugin_id, enabled)
}

pub fn set_plugin_enabled_at(
    path: impl AsRef<Path>,
    plugin_id: &str,
    enabled: bool,
) -> io::Result<()> {
    let path = path.as_ref();
    let mut store = read_plugin_enablement_store(path).unwrap_or_default();
    if enabled {
        store.disabled_plugin_ids.remove(plugin_id);
    } else {
        store.disabled_plugin_ids.insert(plugin_id.to_string());
    }
    write_plugin_enablement_store(path, &store)
}

pub fn plugin_enablement_store_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("addzero")
        .join("az-aio")
        .join(PLUGIN_STATE_FILE)
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginEnablementStore {
    #[serde(default)]
    pub disabled_plugin_ids: BTreeSet<String>,
}

impl PluginEnablementStore {
    pub fn plugin_enabled(&self, plugin_id: &str) -> bool {
        !self.disabled_plugin_ids.contains(plugin_id)
    }
}

#[derive(Clone, Default)]
pub struct HostSnapshot {
    pub admin_menu_tree: AdminMenuTree,
    pub admin_resources: Vec<AdminResourceContract>,
    pub admin_cli: Vec<AdminCliContribution>,
    pub nav_items: Vec<NavItemContribution>,
    pub pages: Vec<PageContribution>,
    pub client_pages: Vec<ClientPageContribution>,
    pub ui_contributions: Vec<UiContribution>,
    pub backend_apis: Vec<BackendApiContribution>,
    pub toolbar_actions: Vec<ToolbarActionContribution>,
    pub catalog_items: Vec<CatalogItemContribution>,
    pub settings_sections: Vec<SettingsSectionContribution>,
    pub shell_entries: Vec<ShellEntryContribution>,
    pub generated_files: Vec<GeneratedFileContribution>,
    pub plugin_contributions: Vec<PluginContributionRecord>,
    pub plugins: Vec<PluginRuntimeRecord>,
    pub native_renderers: Vec<NativeUiRenderer>,
    pub native_router: axum::Router,
}

pub fn client_bootstrap_payload(
    snapshot: &HostSnapshot,
    default_route: impl Into<String>,
    api_base_url: impl Into<String>,
) -> ClientBootstrapPayload {
    ClientBootstrapPayload {
        admin_menu_tree: snapshot.admin_menu_tree.clone(),
        pages: snapshot.pages.clone(),
        client_pages: snapshot.client_pages.clone(),
        plugins: snapshot
            .plugins
            .iter()
            .filter(|record| matches!(record.state, PluginState::Active | PluginState::Loaded))
            .map(|record| ClientPluginRecord {
                descriptor: record.descriptor.clone(),
                state: record.state.clone(),
            })
            .collect(),
        default_route: default_route.into(),
        api_base_url: api_base_url.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginRuntimeRecord {
    pub descriptor: PluginDescriptor,
    pub state: PluginState,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PluginContributionRecord {
    pub plugin_id: String,
    pub contributions: ContributionSet,
}

// ── internal helpers ────────────────────────────────────────────

fn first_duplicate_backend_route(
    apis: &[BackendApiContribution],
    seen_routes: &mut HashSet<(String, String)>,
) -> Option<(String, String)> {
    for api in apis {
        let key = (api.method.clone(), api.path.clone());
        if !seen_routes.insert(key.clone()) {
            return Some(key);
        }
    }
    None
}

fn merge_snapshot_contributions(snapshot: &mut HostSnapshot, contributions: ContributionSet) {
    snapshot.nav_items.extend(contributions.nav_items);
    snapshot.pages.extend(contributions.pages);
    snapshot.client_pages.extend(contributions.client_pages);
    snapshot
        .ui_contributions
        .extend(contributions.ui_contributions);
    snapshot.backend_apis.extend(contributions.backend_apis);
    snapshot
        .toolbar_actions
        .extend(contributions.toolbar_actions);
    snapshot
        .settings_sections
        .extend(contributions.settings_sections);
    snapshot.shell_entries.extend(contributions.shell_entries);
    snapshot
        .generated_files
        .extend(contributions.generated_files);
    for provider in contributions.catalog_providers {
        snapshot.catalog_items.extend(provider.items);
    }
}

fn merge_snapshot_admin_contribution(
    snapshot: &mut HostSnapshot,
    contribution: crate::plugin::contract::AdminPluginContribution,
) {
    merge_menu_tree(&mut snapshot.admin_menu_tree, contribution.menu);
    snapshot.admin_resources.extend(contribution.resources);
    snapshot.admin_cli.extend(contribution.cli);
}

fn sort_snapshot(snapshot: &mut HostSnapshot) {
    let mut contributions = ContributionSet {
        nav_items: std::mem::take(&mut snapshot.nav_items),
        pages: std::mem::take(&mut snapshot.pages),
        client_pages: std::mem::take(&mut snapshot.client_pages),
        ui_contributions: std::mem::take(&mut snapshot.ui_contributions),
        backend_apis: std::mem::take(&mut snapshot.backend_apis),
        toolbar_actions: std::mem::take(&mut snapshot.toolbar_actions),
        catalog_providers: Vec::new(),
        settings_sections: std::mem::take(&mut snapshot.settings_sections),
        shell_entries: std::mem::take(&mut snapshot.shell_entries),
        generated_files: std::mem::take(&mut snapshot.generated_files),
    };
    sort_contributions(&mut contributions);
    snapshot.nav_items = contributions.nav_items;
    snapshot.pages = contributions.pages;
    snapshot.client_pages = contributions.client_pages;
    snapshot.ui_contributions = contributions.ui_contributions;
    snapshot.backend_apis = contributions.backend_apis;
    snapshot.toolbar_actions = contributions.toolbar_actions;
    snapshot.settings_sections = contributions.settings_sections;
    snapshot.shell_entries = contributions.shell_entries;
    snapshot.generated_files = contributions.generated_files;
    snapshot
        .admin_resources
        .sort_by(|left, right| left.route.cmp(&right.route).then(left.id.cmp(&right.id)));
    snapshot.admin_cli.sort_by(|left, right| {
        left.command
            .cmp(&right.command)
            .then(left.id.cmp(&right.id))
    });
    snapshot
        .catalog_items
        .extend(plugin_catalog_items(&snapshot.plugins));
    snapshot.catalog_items.sort_by(|left, right| {
        left.kind
            .label()
            .cmp(right.kind.label())
            .then(left.section.cmp(&right.section))
            .then(left.name.cmp(&right.name))
            .then(left.id.cmp(&right.id))
    });
    snapshot
        .plugin_contributions
        .sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
    snapshot
        .plugins
        .sort_by(|left, right| left.descriptor.id.cmp(&right.descriptor.id));
    snapshot.native_renderers.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.renderer_id.cmp(&right.renderer_id))
    });
}

fn failed_record(descriptor: PluginDescriptor, error: String) -> PluginRuntimeRecord {
    PluginRuntimeRecord {
        descriptor,
        state: PluginState::Failed,
        error: Some(error),
    }
}

fn disabled_record(descriptor: PluginDescriptor) -> PluginRuntimeRecord {
    PluginRuntimeRecord {
        descriptor,
        state: PluginState::Disabled,
        error: None,
    }
}

fn sort_contributions(contributions: &mut ContributionSet) {
    contributions
        .nav_items
        .sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
    contributions.pages.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.route.cmp(&right.route))
    });
    contributions.client_pages.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.route.cmp(&right.route))
            .then(left.renderer_id.cmp(&right.renderer_id))
    });
    contributions.ui_contributions.sort_by(|left, right| {
        left.slot
            .label()
            .cmp(right.slot.label())
            .then(left.route.cmp(&right.route))
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    contributions.backend_apis.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.method.cmp(&right.method))
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    contributions.toolbar_actions.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    contributions
        .catalog_providers
        .sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
    contributions
        .settings_sections
        .sort_by(|left, right| left.order.cmp(&right.order).then(left.id.cmp(&right.id)));
    contributions.shell_entries.sort_by(|left, right| {
        left.kind
            .label()
            .cmp(right.kind.label())
            .then(left.section.cmp(&right.section))
            .then(left.name.cmp(&right.name))
            .then(left.source_path.cmp(&right.source_path))
            .then(left.line_start.cmp(&right.line_start))
    });
    contributions.generated_files.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.source_root.cmp(&right.source_root))
    });
}

fn plugin_catalog_items(records: &[PluginRuntimeRecord]) -> Vec<CatalogItemContribution> {
    records
        .iter()
        .map(|record| CatalogItemContribution {
            id: record.descriptor.id.clone(),
            name: record.descriptor.name.clone(),
            description: record
                .error
                .clone()
                .unwrap_or_else(|| record.descriptor.description.clone()),
            section: "插件".to_string(),
            icon: "◇".to_string(),
            accent_class: match record.state {
                PluginState::Failed => "plugin-icon--git",
                _ => "plugin-icon--automation",
            }
            .to_string(),
            kind: CatalogItemKind::Plugin,
            source: CatalogSource::Bundled,
            installed: record.state == PluginState::Active || record.state == PluginState::Loaded,
            tags: Vec::new(),
            permissions: record.descriptor.permissions.clone(),
            path: None,
        })
        .collect()
}

pub fn descriptor(
    id: &str,
    name: &str,
    description: &str,
    priority: i32,
    dependencies: Vec<crate::plugin::contract::PluginDependency>,
    capabilities: Vec<&str>,
) -> PluginDescriptor {
    PluginDescriptor {
        id: id.to_string(),
        name: name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: description.to_string(),
        activation: PluginActivation::Eager,
        priority,
        dependencies,
        capabilities: capabilities.into_iter().map(str::to_string).collect(),
        permissions: Vec::new(),
        kind: PluginKind::Native,
    }
}

// ── plugin enablement persistence ─────────────────────────────────

fn read_plugin_enablement_store(path: &Path) -> io::Result<PluginEnablementStore> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid plugin state file: {error}"),
            )
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(PluginEnablementStore::default())
        }
        Err(error) => Err(error),
    }
}

fn write_plugin_enablement_store(path: &Path, store: &PluginEnablementStore) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_string_pretty(store).map_err(io::Error::other)?;
    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::plugin::contract::{
        AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminPluginContribution,
        AdminPluginProvider, BackendApiContribution, ClientPageContribution, NativePluginRuntime,
        PageContribution, UiContributionSlot,
    };

    use super::*;

    #[derive(Clone)]
    struct TestProvider {
        id: &'static str,
        route: &'static str,
        api_path: &'static str,
    }

    impl AdminPluginProvider for TestProvider {
        fn admin_descriptor(&self) -> PluginDescriptor {
            descriptor(self.id, self.id, "测试插件", 10, Vec::new(), vec!["test"])
        }

        fn admin_contribution(&self) -> anyhow::Result<AdminPluginContribution> {
            Ok(AdminPluginContribution {
                menu: AdminMenuTree {
                    sections: vec![AdminMenuSection {
                        domain_id: "test".to_string(),
                        label: "测试".to_string(),
                        default_href: self.route.to_string(),
                        order: 10,
                        menus: vec![AdminMenuNode {
                            id: format!("{}.nav", self.id),
                            kind: AdminMenuNodeKind::Page,
                            label: self.id.to_string(),
                            href: self.route.to_string(),
                            icon: "T".to_string(),
                            order: 10,
                            active_patterns: vec![self.route.to_string()],
                            permissions_any_of: Vec::new(),
                            children: Vec::new(),
                        }],
                    }],
                },
                resources: Vec::new(),
                cli: Vec::new(),
                native: ContributionSet {
                    pages: vec![PageContribution {
                        route: self.route.to_string(),
                        title: self.id.to_string(),
                        subtitle: "测试页面".to_string(),
                        renderer_id: format!("{}.page", self.id),
                        placeholder_mark: "T".to_string(),
                        order: 10,
                    }],
                    client_pages: vec![ClientPageContribution {
                        route: self.route.to_string(),
                        title: self.id.to_string(),
                        renderer_id: format!("{}.page", self.id),
                        slot: UiContributionSlot::Content,
                        order: 10,
                    }],
                    backend_apis: vec![BackendApiContribution {
                        id: format!("{}.api", self.id),
                        method: "GET".to_string(),
                        path: self.api_path.to_string(),
                        label: "测试 API".to_string(),
                        description: "测试 API".to_string(),
                        order: 10,
                    }],
                    ..ContributionSet::default()
                },
            })
        }

        fn admin_runtime(
            &self,
            _context: NativePluginContext,
        ) -> anyhow::Result<NativePluginRuntime> {
            Ok(NativePluginRuntime::default())
        }
    }

    #[test]
    fn empty_host_does_not_create_business_menu() {
        let snapshot = NativePluginHost::new(NativePluginContext::default()).load_snapshot();

        assert!(snapshot.admin_menu_tree.sections.is_empty());
        assert!(snapshot.nav_items.is_empty());
        assert!(snapshot.pages.is_empty());
    }

    #[test]
    fn host_menu_comes_from_admin_provider() {
        let provider = Arc::new(TestProvider {
            id: "demo",
            route: "/demo",
            api_path: "/api/demo",
        });
        let snapshot = NativePluginHost::new(NativePluginContext::default())
            .with_plugin(provider)
            .load_snapshot();

        assert_eq!(snapshot.admin_menu_tree.sections[0].default_href, "/demo");
        assert_eq!(snapshot.backend_apis[0].path, "/api/demo");
    }

    #[test]
    fn duplicate_backend_route_marks_second_plugin_failed() {
        let first = Arc::new(TestProvider {
            id: "first",
            route: "/first",
            api_path: "/api/shared",
        });
        let second = Arc::new(TestProvider {
            id: "second",
            route: "/second",
            api_path: "/api/shared",
        });

        let snapshot = NativePluginHost::new(NativePluginContext::default())
            .with_plugin(first)
            .with_plugin(second)
            .load_snapshot();
        let failed = snapshot
            .plugins
            .iter()
            .filter(|record| record.state == PluginState::Failed)
            .count();

        assert_eq!(failed, 1);
        assert_eq!(snapshot.backend_apis.len(), 1);
    }

    #[test]
    fn client_bootstrap_excludes_disabled_plugins() {
        let enabled = Arc::new(TestProvider {
            id: "enabled",
            route: "/enabled",
            api_path: "/api/enabled",
        });
        let disabled = Arc::new(TestProvider {
            id: "disabled",
            route: "/disabled",
            api_path: "/api/disabled",
        });
        let enablement = PluginEnablementStore {
            disabled_plugin_ids: ["disabled".to_string()].into_iter().collect(),
        };

        let snapshot = NativePluginHost::new(NativePluginContext::default())
            .with_plugin(enabled)
            .with_plugin(disabled)
            .load_snapshot_with_enablement(&enablement);
        let payload = client_bootstrap_payload(&snapshot, "/enabled", "");
        let plugin_ids = payload
            .plugins
            .iter()
            .map(|record| record.descriptor.id.as_str())
            .collect::<Vec<_>>();

        // Disabled plugins must not be discoverable by the browser bootstrap payload.
        assert_eq!(plugin_ids, ["enabled"]);
    }
}
