use std::{
    fs,
    time::{Duration, Instant},
};

use az_aio_plugin_api::{
    AzAioPlugin, BackendApiContribution, CatalogItemKind, ContributionSet, NativeAzAioPlugin,
    NativePluginContext, NativePluginRuntime, NavItemContribution, PluginDependency, PluginState,
    UiContribution, UiContributionSlot,
};
use az_aio_plugin_host::host::{
    NativePluginHost, PluginEnablementStore, PluginHost, descriptor, set_plugin_enabled_at,
};
use tempfile::TempDir;

#[derive(Default)]
struct TestPlugin {
    id: &'static str,
    priority: i32,
    dependencies: Vec<PluginDependency>,
    fail_enable: bool,
    enable_delay: Duration,
    nav_label: Option<&'static str>,
    ui_contribution: Option<UiContribution>,
    backend_api: Option<BackendApiContribution>,
}

impl TestPlugin {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            priority: 0,
            dependencies: Vec::new(),
            fail_enable: false,
            enable_delay: Duration::ZERO,
            nav_label: None,
            ui_contribution: None,
            backend_api: None,
        }
    }
}

impl AzAioPlugin for TestPlugin {
    fn descriptor(&self) -> az_aio_plugin_api::PluginDescriptor {
        descriptor(
            self.id,
            self.id,
            "test plugin",
            self.priority,
            self.dependencies.clone(),
            vec!["test"],
        )
    }

    fn on_enable(&mut self) -> anyhow::Result<()> {
        if !self.enable_delay.is_zero() {
            std::thread::sleep(self.enable_delay);
        }
        if self.fail_enable {
            Err(anyhow::anyhow!("enable failed"))
        } else {
            Ok(())
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        let mut set = ContributionSet::default();
        if let Some(label) = self.nav_label {
            set.nav_items.push(NavItemContribution {
                id: self.id.to_string(),
                label: label.to_string(),
                icon: "•".to_string(),
                route: format!("/{label}"),
                order: self.priority,
            });
        }
        if let Some(ui_contribution) = self.ui_contribution.clone() {
            set.ui_contributions.push(ui_contribution);
        }
        if let Some(backend_api) = self.backend_api.clone() {
            set.backend_apis.push(backend_api);
        }
        Ok(set)
    }
}

#[derive(Default)]
struct TestNativePlugin {
    id: &'static str,
    route: &'static str,
}

impl NativeAzAioPlugin for TestNativePlugin {
    fn descriptor(&self) -> az_aio_plugin_api::PluginDescriptor {
        let mut descriptor = descriptor(
            self.id,
            self.id,
            "native test plugin",
            0,
            vec![],
            vec!["native"],
        );
        descriptor.kind = az_aio_plugin_api::PluginKind::Native;
        descriptor
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            backend_apis: vec![BackendApiContribution {
                id: format!("{}.api.status", self.id),
                method: "GET".to_string(),
                path: self.route.to_string(),
                label: "Status".to_string(),
                description: "Native test status.".to_string(),
                order: 10,
            }],
            ..ContributionSet::default()
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

#[test]
fn duplicate_plugin_ids_are_rejected_without_crashing_host() {
    let snapshot = PluginHost::new()
        .with_plugin(Box::new(TestPlugin::new("same")))
        .with_plugin(Box::new(TestPlugin::new("same")))
        .load_snapshot();

    let failed = snapshot
        .plugins
        .iter()
        .filter(|plugin| plugin.state == PluginState::Failed)
        .count();
    assert_eq!(failed, 1);
}

#[test]
fn dependencies_are_ordered_before_priority_and_id() {
    let mut dependent = TestPlugin::new("dependent");
    dependent.priority = 100;
    dependent.nav_label = Some("dependent");
    dependent.dependencies = vec![PluginDependency {
        id: "base".to_string(),
        optional: false,
    }];
    let mut base = TestPlugin::new("base");
    base.priority = 1;
    base.nav_label = Some("base");

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(dependent))
        .with_plugin(Box::new(base))
        .load_snapshot();

    let labels = snapshot
        .nav_items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["base", "dependent"]);
}

#[test]
fn present_optional_dependency_is_loaded_before_dependent_plugin() {
    let mut dependent = TestPlugin::new("dependent");
    dependent.priority = 100;
    dependent.nav_label = Some("dependent");
    dependent.dependencies = vec![PluginDependency {
        id: "settings".to_string(),
        optional: true,
    }];
    let mut settings = TestPlugin::new("settings");
    settings.priority = 1;
    settings.nav_label = Some("settings");

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(dependent))
        .with_plugin(Box::new(settings))
        .load_snapshot();

    let labels = snapshot
        .nav_items
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["settings", "dependent"]);
}

#[test]
fn missing_optional_dependency_does_not_block_plugin_activation() {
    let mut plugin = TestPlugin::new("sync");
    plugin.dependencies = vec![PluginDependency {
        id: "settings".to_string(),
        optional: true,
    }];

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(plugin))
        .load_snapshot();

    assert!(
        snapshot.plugins.iter().any(|plugin| {
            plugin.descriptor.id == "sync" && plugin.state == PluginState::Active
        })
    );
}

#[test]
fn independent_plugins_activate_in_parallel_within_same_dependency_layer() {
    let mut alpha = TestPlugin::new("alpha");
    alpha.enable_delay = Duration::from_millis(250);
    let mut beta = TestPlugin::new("beta");
    beta.enable_delay = Duration::from_millis(250);

    let started = Instant::now();
    let snapshot = PluginHost::new()
        .with_plugin(Box::new(alpha))
        .with_plugin(Box::new(beta))
        .load_snapshot();

    assert!(
        snapshot
            .plugins
            .iter()
            .all(|plugin| { plugin.state == PluginState::Active })
    );
    assert!(
        started.elapsed() < Duration::from_millis(450),
        "same-layer plugin activation should run concurrently"
    );
}

#[test]
fn dependent_plugin_does_not_activate_when_required_dependency_fails() {
    let mut base = TestPlugin::new("settings");
    base.fail_enable = true;
    let mut dependent = TestPlugin::new("sync");
    dependent.dependencies = vec![PluginDependency {
        id: "settings".to_string(),
        optional: false,
    }];

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(base))
        .with_plugin(Box::new(dependent))
        .load_snapshot();

    assert!(snapshot.plugins.iter().any(|plugin| {
        plugin.descriptor.id == "settings" && plugin.state == PluginState::Failed
    }));
    let sync = snapshot
        .plugins
        .iter()
        .find(|plugin| plugin.descriptor.id == "sync")
        .expect("dependent plugin record");
    assert_eq!(sync.state, PluginState::Failed);
    assert!(
        sync.error
            .as_deref()
            .is_some_and(|error| error.contains("依赖 `settings` 未成功加载"))
    );
}

#[test]
fn failed_lifecycle_is_exposed_but_other_plugins_stay_active() {
    let mut failing = TestPlugin::new("failing");
    failing.fail_enable = true;
    let healthy = TestPlugin::new("healthy");

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(failing))
        .with_plugin(Box::new(healthy))
        .load_snapshot();

    assert!(snapshot.plugins.iter().any(|plugin| {
        plugin.descriptor.id == "failing" && plugin.state == PluginState::Failed
    }));
    assert!(snapshot.plugins.iter().any(|plugin| {
        plugin.descriptor.id == "healthy" && plugin.state == PluginState::Active
    }));
}

#[test]
fn descriptor_contributions_are_aggregated() {
    let mut alpha = TestPlugin::new("alpha");
    alpha.priority = 1;
    alpha.nav_label = Some("alpha");
    let mut beta = TestPlugin::new("beta");
    beta.priority = 2;
    beta.nav_label = Some("beta");

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(alpha))
        .with_plugin(Box::new(beta))
        .load_snapshot();

    assert_eq!(snapshot.nav_items.len(), 2);
}

#[test]
fn ui_and_backend_api_contributions_are_aggregated() {
    let mut plugin = TestPlugin::new("projects");
    plugin.ui_contribution = Some(UiContribution {
        id: "projects.ui.sidebar".to_string(),
        slot: UiContributionSlot::ProjectSidebar,
        label: "项目侧边栏".to_string(),
        renderer_id: "projects.sidebar".to_string(),
        route: Some("/projects".to_string()),
        order: 10,
    });
    plugin.backend_api = Some(BackendApiContribution {
        id: "projects.api.list".to_string(),
        method: "GET".to_string(),
        path: "/api/projects".to_string(),
        label: "项目列表".to_string(),
        description: "列出已绑定项目。".to_string(),
        order: 10,
    });

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(plugin))
        .load_snapshot();

    assert_eq!(snapshot.ui_contributions[0].id, "projects.ui.sidebar");
    assert_eq!(snapshot.backend_apis[0].id, "projects.api.list");
    assert_eq!(snapshot.plugin_contributions[0].plugin_id, "projects");
    assert_eq!(
        snapshot.plugin_contributions[0]
            .contributions
            .ui_contributions[0]
            .id,
        "projects.ui.sidebar"
    );
}

#[test]
fn invalid_wasm_components_are_cataloged_as_failed_plugins() {
    let temp = TempDir::new().expect("create temp dir");
    let wasm_path = temp.path().join("broken.wasm");
    fs::write(&wasm_path, b"not a component").expect("write invalid wasm");

    let snapshot = PluginHost::new()
        .with_wasm_components_from_dir(temp.path())
        .load_snapshot();

    assert!(snapshot.plugins.iter().any(|plugin| {
        plugin.descriptor.kind == az_aio_plugin_api::PluginKind::WasmComponent
            && plugin.state == PluginState::Failed
    }));
    assert!(
        snapshot.catalog_items.iter().any(|item| {
            item.source == az_aio_plugin_api::CatalogSource::Wasm && !item.installed
        })
    );
}

#[test]
fn native_host_rejects_duplicate_backend_routes() {
    let snapshot = NativePluginHost::new(NativePluginContext::default())
        .with_plugin(Box::new(TestNativePlugin {
            id: "alpha",
            route: "/api/native/status",
        }))
        .with_plugin(Box::new(TestNativePlugin {
            id: "beta",
            route: "/api/native/status",
        }))
        .load_snapshot();

    assert!(
        snapshot.plugins.iter().any(|plugin| {
            plugin.descriptor.id == "beta" && plugin.state == PluginState::Failed
        })
    );
    assert_eq!(snapshot.backend_apis.len(), 1);
}

#[test]
fn native_inventory_host_can_load_without_panic() {
    let snapshot = NativePluginHost::from_inventory(NativePluginContext::default()).load_snapshot();

    assert!(
        snapshot
            .plugins
            .iter()
            .all(|plugin| { plugin.descriptor.kind == az_aio_plugin_api::PluginKind::Native })
    );
}

#[test]
fn disabled_plugin_is_cataloged_without_contributions() {
    let mut enablement = PluginEnablementStore::default();
    enablement
        .disabled_plugin_ids
        .insert("settings".to_string());

    let mut plugin = TestPlugin::new("settings");
    plugin.ui_contribution = Some(UiContribution {
        id: "settings.ui.project-defaults".to_string(),
        slot: UiContributionSlot::SettingsContent,
        label: "项目默认目录".to_string(),
        renderer_id: "settings.project-defaults".to_string(),
        route: Some("/settings".to_string()),
        order: 10,
    });
    plugin.backend_api = Some(BackendApiContribution {
        id: "settings.api.project-defaults".to_string(),
        method: "GET".to_string(),
        path: "/api/settings/project-defaults".to_string(),
        label: "项目默认设置".to_string(),
        description: "读取默认设置。".to_string(),
        order: 10,
    });

    let snapshot = PluginHost::new()
        .with_plugin(Box::new(plugin))
        .load_snapshot_with_enablement(&enablement);

    assert!(snapshot.plugins.iter().any(|plugin| {
        plugin.descriptor.id == "settings" && plugin.state == PluginState::Disabled
    }));
    assert!(snapshot.catalog_items.iter().any(|item| {
        item.id == "settings" && item.kind == CatalogItemKind::Plugin && !item.installed
    }));
    assert!(
        !snapshot
            .ui_contributions
            .iter()
            .any(|item| item.id == "settings.ui.project-defaults")
    );
    assert!(
        !snapshot
            .backend_apis
            .iter()
            .any(|item| item.id == "settings.api.project-defaults")
    );
}

#[test]
fn plugin_enablement_store_persists_disabled_ids() {
    let temp = TempDir::new().expect("create temp dir");
    let path = temp.path().join("plugin-state.json");

    set_plugin_enabled_at(&path, "settings", false).expect("disable plugin");
    let stored = fs::read_to_string(&path).expect("read plugin state");
    let store: PluginEnablementStore = serde_json::from_str(&stored).expect("parse plugin state");
    assert!(store.disabled_plugin_ids.contains("settings"));

    set_plugin_enabled_at(&path, "settings", true).expect("enable plugin");
    let stored = fs::read_to_string(&path).expect("read plugin state");
    let store: PluginEnablementStore = serde_json::from_str(&stored).expect("parse plugin state");
    assert!(!store.disabled_plugin_ids.contains("settings"));
}
