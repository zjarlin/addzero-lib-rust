use std::{
    collections::{BTreeSet, HashSet},
    fs, io,
    path::{Path, PathBuf},
    thread,
};

use az_aio_plugin_api::api::{
    BackendApiContribution, CatalogItemContribution, CatalogItemKind, CatalogSource,
    ContributionSet, GeneratedFileContribution, NavItemContribution, PageContribution,
    PluginActivation, PluginDependency, PluginDescriptor, PluginKind, PluginState,
    SettingsSectionContribution, ShellEntryContribution, ToolbarActionContribution, UiContribution,
};
#[cfg(not(target_arch = "wasm32"))]
use az_aio_plugin_api::api::{
    NativeAzAioPlugin, NativePluginContext, NativePluginRegistration, NativeRenderFn,
    NativeUiRenderer,
};
use serde::{Deserialize, Serialize};

const PLUGIN_STATE_FILE: &str = "plugin-state.json";

#[cfg(not(target_arch = "wasm32"))]
pub fn load_az_aio_native_snapshot(context: NativePluginContext) -> HostSnapshot {
    let enablement = load_plugin_enablement();
    NativePluginHost::from_inventory(context)
        .load_snapshot_with_enablement(&enablement)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn native_renderer(snapshot: &HostSnapshot, renderer_id: &str) -> Option<NativeRenderFn> {
    snapshot
        .native_renderers
        .iter()
        .find(|renderer| renderer.renderer_id == renderer_id)
        .map(|renderer| renderer.render)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn start_native_loopback_server(snapshot: HostSnapshot) -> anyhow::Result<String> {
    let app = snapshot.native_router.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let local_addr = listener.local_addr()?;
    thread::Builder::new()
        .name("az-aio-native-plugin-server".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("az-aio native plugin runtime failed: {error}");
                    return;
                }
            };
            runtime.block_on(async move {
                match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => {
                        if let Err(error) = axum::serve(listener, app).await {
                            eprintln!("az-aio native plugin server failed: {error}");
                        }
                    }
                    Err(error) => {
                        eprintln!("az-aio native plugin listener failed: {error}");
                    }
                }
            });
        })?;
    Ok(format!("http://{local_addr}"))
}

#[cfg(not(target_arch = "wasm32"))]
pub struct NativePluginHost {
    plugins: Vec<Box<dyn NativeAzAioPlugin>>,
    context: NativePluginContext,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativePluginHost {
    pub fn new(context: NativePluginContext) -> Self {
        Self {
            plugins: Vec::new(),
            context,
        }
    }

    pub fn from_inventory(context: NativePluginContext) -> Self {
        let mut plugins = inventory::iter::<NativePluginRegistration>
            .into_iter()
            .map(|registration| (registration.constructor)())
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.descriptor().id.cmp(&right.descriptor().id));
        Self { plugins, context }
    }

    pub fn with_plugin(mut self, plugin: Box<dyn NativeAzAioPlugin>) -> Self {
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
            let descriptor = plugin.descriptor();
            if !seen_ids.insert(descriptor.id.clone()) {
                snapshot.plugins.push(failed_record(
                    descriptor.clone(),
                    format!("插件 ID 重复：{}", descriptor.id),
                ));
                continue;
            }
            if !enablement.plugin_enabled(&descriptor.id) {
                snapshot.plugins.push(disabled_record(descriptor));
                continue;
            }

            let contributions = match plugin.contributions() {
                Ok(c) => c,
                Err(error) => {
                    snapshot.plugins.push(failed_record(
                        descriptor.clone(),
                        format!(
                            "插件 `{}` 在 native-contributions 阶段失败：{}",
                            descriptor.id, error
                        ),
                    ));
                    continue;
                }
            };

            let runtime = match plugin.runtime(self.context.clone()) {
                Ok(r) => r,
                Err(error) => {
                    snapshot.plugins.push(failed_record(
                        descriptor.clone(),
                        format!(
                            "插件 `{}` native runtime 阶段失败：{}",
                            descriptor.id, error
                        ),
                    ));
                    continue;
                }
            };

            if let Some(startup) = runtime.startup {
                if let Err(error) = startup(self.context.clone()) {
                    snapshot.plugins.push(failed_record(
                        descriptor.clone(),
                        format!(
                            "插件 `{}` native startup 阶段失败：{}",
                            descriptor.id, error
                        ),
                    ));
                    continue;
                }
            }

            if let Some((method, path)) =
                first_duplicate_backend_route(&contributions.backend_apis, &mut seen_routes)
            {
                snapshot.plugins.push(failed_record(
                    descriptor.clone(),
                    format!("native backend route duplicated: {method} {path}"),
                ));
                continue;
            }

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
        snapshot
    }
}

pub fn load_plugin_enablement() -> PluginEnablementStore {
    read_plugin_enablement_store(&plugin_enablement_store_path()).unwrap_or_default()
}

pub fn set_plugin_enabled(plugin_id: &str, enabled: bool) -> io::Result<()> {
    set_plugin_enabled_at(&plugin_enablement_store_path(), plugin_id, enabled)
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
        .unwrap_or_else(|| az_io::home_dir::home_dir().join(".config"))
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
    pub nav_items: Vec<NavItemContribution>,
    pub pages: Vec<PageContribution>,
    pub ui_contributions: Vec<UiContribution>,
    pub backend_apis: Vec<BackendApiContribution>,
    pub toolbar_actions: Vec<ToolbarActionContribution>,
    pub catalog_items: Vec<CatalogItemContribution>,
    pub settings_sections: Vec<SettingsSectionContribution>,
    pub shell_entries: Vec<ShellEntryContribution>,
    pub generated_files: Vec<GeneratedFileContribution>,
    pub plugin_contributions: Vec<PluginContributionRecord>,
    pub plugins: Vec<PluginRuntimeRecord>,
    #[cfg(not(target_arch = "wasm32"))]
    pub native_renderers: Vec<NativeUiRenderer>,
    #[cfg(not(target_arch = "wasm32"))]
    pub native_router: axum::Router,
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

// ── native loading helpers ────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn merge_snapshot_contributions(snapshot: &mut HostSnapshot, contributions: ContributionSet) {
    snapshot.nav_items.extend(contributions.nav_items);
    snapshot.pages.extend(contributions.pages);
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

#[cfg(not(target_arch = "wasm32"))]
fn sort_snapshot(snapshot: &mut HostSnapshot) {
    let mut contributions = ContributionSet {
        nav_items: std::mem::take(&mut snapshot.nav_items),
        pages: std::mem::take(&mut snapshot.pages),
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
    snapshot.ui_contributions = contributions.ui_contributions;
    snapshot.backend_apis = contributions.backend_apis;
    snapshot.toolbar_actions = contributions.toolbar_actions;
    snapshot.settings_sections = contributions.settings_sections;
    snapshot.shell_entries = contributions.shell_entries;
    snapshot.generated_files = contributions.generated_files;
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
    dependencies: Vec<PluginDependency>,
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

// ── plugin enablement persistence ─────────────────────────────────────

fn read_plugin_enablement_store(path: &Path) -> io::Result<PluginEnablementStore> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("插件状态文件格式无效：{error}"),
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

