#![forbid(unsafe_code)]

mod wasm_component;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
};

use az_aio_plugin_api::{
    AzAioPlugin, BackendApiContribution, CatalogItemContribution, CatalogItemKind, CatalogSource,
    ContributionSet, GeneratedFileContribution, NavItemContribution, PageContribution,
    PluginActivation, PluginDependency, PluginDescriptor, PluginError, PluginKind, PluginState,
    SettingsSectionContribution, ShellEntryContribution, ToolbarActionContribution, UiContribution,
};
use az_aio_plugin_catalog::CatalogPlugin;
use az_aio_plugin_core_nav::CoreNavPlugin;
use az_aio_plugin_git_clis::GitClisPlugin;
use az_aio_plugin_git_envs::GitEnvsPlugin;
use az_aio_plugin_git_notes::GitNotesPlugin;
use az_aio_plugin_git_skills::GitSkillsPlugin;
use az_aio_plugin_projects::ProjectsPlugin;
use az_aio_plugin_search::SearchPlugin;
use az_aio_plugin_settings::SettingsPlugin;

pub use wasm_component::WasmComponentPlugin;

pub struct PluginHost {
    plugins: Vec<Box<dyn AzAioPlugin>>,
    records: Vec<PluginRuntimeRecord>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn with_plugin(mut self, plugin: Box<dyn AzAioPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn with_plugins(mut self, plugins: impl IntoIterator<Item = Box<dyn AzAioPlugin>>) -> Self {
        self.plugins.extend(plugins);
        self
    }

    pub fn with_builtin_plugins() -> Self {
        Self::new().with_plugins([
            Box::<CoreNavPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<SettingsPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<SearchPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<ProjectsPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<CatalogPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<GitSkillsPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<GitClisPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<GitEnvsPlugin>::default() as Box<dyn AzAioPlugin>,
            Box::<GitNotesPlugin>::default() as Box<dyn AzAioPlugin>,
        ])
    }

    pub fn with_wasm_components_from_dir(mut self, dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        let Ok(entries) = fs::read_dir(dir) else {
            return self;
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !is_wasm_component_file(&path) {
                continue;
            }

            match WasmComponentPlugin::from_file(&path) {
                Ok(plugin) => self.plugins.push(Box::new(plugin)),
                Err(error) => self.records.push(failed_record(
                    failed_wasm_descriptor(&path),
                    error.to_string(),
                )),
            }
        }

        self
    }

    pub fn load_snapshot(self) -> HostSnapshot {
        HostLoader::new(self.plugins, self.records).load()
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_az_aio_plugin_snapshot() -> HostSnapshot {
    PluginHost::with_builtin_plugins().load_snapshot()
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

struct PluginSlot {
    plugin: Box<dyn AzAioPlugin>,
    descriptor: PluginDescriptor,
}

struct HostLoader {
    slots: Vec<PluginSlot>,
    records: Vec<PluginRuntimeRecord>,
    contributions: ContributionSet,
    plugin_contributions: Vec<PluginContributionRecord>,
}

impl HostLoader {
    fn new(plugins: Vec<Box<dyn AzAioPlugin>>, records: Vec<PluginRuntimeRecord>) -> Self {
        let mut seen = records
            .iter()
            .map(|record| record.descriptor.id.clone())
            .collect::<HashSet<_>>();
        let mut slots = Vec::new();
        let mut records = records;

        for plugin in plugins {
            let descriptor = plugin.descriptor();
            if seen.insert(descriptor.id.clone()) {
                slots.push(PluginSlot { plugin, descriptor });
            } else {
                records.push(failed_record(
                    descriptor.clone(),
                    PluginError::DuplicateId(descriptor.id.clone()).to_string(),
                ));
            }
        }

        Self {
            slots,
            records,
            contributions: ContributionSet::default(),
            plugin_contributions: Vec::new(),
        }
    }

    fn load(mut self) -> HostSnapshot {
        let (order, failures) = activation_order(&self.slots);
        for (index, error) in failures {
            self.records.push(failed_record(
                self.slots[index].descriptor.clone(),
                error.to_string(),
            ));
        }

        for index in order {
            if self
                .records
                .iter()
                .any(|record| record.descriptor.id == self.slots[index].descriptor.id)
            {
                continue;
            }
            self.activate_slot(index);
        }

        self.snapshot()
    }

    fn activate_slot(&mut self, index: usize) {
        let descriptor = self.slots[index].descriptor.clone();
        let plugin = &mut self.slots[index].plugin;
        if let Err(error) = plugin.on_load() {
            self.records.push(failed_record(
                descriptor.clone(),
                lifecycle_message(&descriptor.id, "on-load", error),
            ));
            return;
        }

        if descriptor.activation == PluginActivation::Eager {
            if let Err(error) = plugin.on_enable() {
                self.records.push(failed_record(
                    descriptor.clone(),
                    lifecycle_message(&descriptor.id, "on-enable", error),
                ));
                return;
            }
        }

        match plugin.contributions() {
            Ok(contributions) => {
                self.plugin_contributions.push(PluginContributionRecord {
                    plugin_id: descriptor.id.clone(),
                    contributions: contributions.clone(),
                });
                self.contributions.merge(contributions);
                let state = if descriptor.activation == PluginActivation::Eager {
                    PluginState::Active
                } else {
                    PluginState::Loaded
                };
                self.records.push(PluginRuntimeRecord {
                    descriptor,
                    state,
                    error: None,
                });
            }
            Err(error) => self.records.push(failed_record(
                descriptor.clone(),
                lifecycle_message(&descriptor.id, "contributions", error),
            )),
        }
    }

    fn snapshot(mut self) -> HostSnapshot {
        sort_contributions(&mut self.contributions);
        for record in &mut self.plugin_contributions {
            sort_contributions(&mut record.contributions);
        }
        self.plugin_contributions
            .sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        let mut catalog_items = plugin_catalog_items(&self.records);
        for provider in self.contributions.catalog_providers.iter() {
            catalog_items.extend(provider.items.clone());
        }
        catalog_items.sort_by(|left, right| {
            left.kind
                .label()
                .cmp(right.kind.label())
                .then(left.section.cmp(&right.section))
                .then(left.name.cmp(&right.name))
                .then(left.id.cmp(&right.id))
        });
        self.records
            .sort_by(|left, right| left.descriptor.id.cmp(&right.descriptor.id));

        HostSnapshot {
            nav_items: self.contributions.nav_items,
            pages: self.contributions.pages,
            ui_contributions: self.contributions.ui_contributions,
            backend_apis: self.contributions.backend_apis,
            toolbar_actions: self.contributions.toolbar_actions,
            catalog_items,
            settings_sections: self.contributions.settings_sections,
            shell_entries: self.contributions.shell_entries,
            generated_files: self.contributions.generated_files,
            plugin_contributions: self.plugin_contributions,
            plugins: self.records,
        }
    }
}

fn lifecycle_message(plugin_id: &str, phase: &str, error: PluginError) -> String {
    PluginError::Lifecycle {
        plugin: plugin_id.to_string(),
        phase: phase.to_string(),
        message: error.to_string(),
    }
    .to_string()
}

fn failed_record(descriptor: PluginDescriptor, error: String) -> PluginRuntimeRecord {
    PluginRuntimeRecord {
        descriptor,
        state: PluginState::Failed,
        error: Some(error),
    }
}

fn is_wasm_component_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wasm"))
}

fn failed_wasm_descriptor(path: &Path) -> PluginDescriptor {
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("外部组件加载失败")
        .to_string();

    PluginDescriptor {
        id: format!("wasm/{}", sanitize_id(&path.display().to_string())),
        name,
        version: "未知".to_string(),
        description: "外部组件描述符发现失败。".to_string(),
        activation: PluginActivation::Eager,
        priority: 0,
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        permissions: vec![format!("读取 {}", path.display())],
        kind: PluginKind::WasmComponent,
    }
}

fn activation_order(slots: &[PluginSlot]) -> (Vec<usize>, Vec<(usize, PluginError)>) {
    let mut failures = Vec::new();
    let id_to_index = slots
        .iter()
        .enumerate()
        .map(|(index, slot)| (slot.descriptor.id.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut failed = BTreeSet::new();

    for (index, slot) in slots.iter().enumerate() {
        for dependency in slot
            .descriptor
            .dependencies
            .iter()
            .filter(|dep| !dep.optional)
        {
            if !id_to_index.contains_key(&dependency.id) {
                failed.insert(index);
                failures.push((
                    index,
                    PluginError::MissingDependency {
                        plugin: slot.descriptor.id.clone(),
                        dependency: dependency.id.clone(),
                    },
                ));
            }
        }
    }

    let mut remaining = slots
        .iter()
        .enumerate()
        .filter_map(|(index, _)| (!failed.contains(&index)).then_some(index))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::new();
    let mut activated_ids = HashSet::new();

    while !remaining.is_empty() {
        let mut ready = remaining
            .iter()
            .copied()
            .filter(|index| {
                dependencies_ready(&slots[*index].descriptor.dependencies, &activated_ids)
            })
            .collect::<Vec<_>>();

        if ready.is_empty() {
            for index in remaining {
                failures.push((
                    index,
                    PluginError::DependencyCycle(slots[index].descriptor.id.clone()),
                ));
            }
            break;
        }

        ready.sort_by(|left, right| {
            slots[*right]
                .descriptor
                .priority
                .cmp(&slots[*left].descriptor.priority)
                .then(slots[*left].descriptor.id.cmp(&slots[*right].descriptor.id))
        });

        for index in ready {
            remaining.remove(&index);
            activated_ids.insert(slots[index].descriptor.id.clone());
            ordered.push(index);
        }
    }

    (ordered, failures)
}

fn dependencies_ready(dependencies: &[PluginDependency], activated_ids: &HashSet<String>) -> bool {
    dependencies
        .iter()
        .filter(|dependency| !dependency.optional)
        .all(|dependency| activated_ids.contains(&dependency.id))
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
            icon: match record.descriptor.kind {
                PluginKind::Native => "⌘".to_string(),
                PluginKind::WasmComponent => "◇".to_string(),
            },
            accent_class: match record.state {
                PluginState::Failed => "plugin-icon--git",
                _ => "plugin-icon--automation",
            }
            .to_string(),
            kind: CatalogItemKind::Plugin,
            source: match record.descriptor.kind {
                PluginKind::Native => CatalogSource::Local,
                PluginKind::WasmComponent => CatalogSource::Wasm,
            },
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

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use az_aio_plugin_api::{
        BackendApiContribution, CatalogItemKind, ContributionSet, NavItemContribution,
        PluginDependency, PluginError, PluginSandboxDebugReport, PluginState, UiContribution,
        UiContributionSlot,
    };
    use tempfile::TempDir;

    use super::{AzAioPlugin, PluginHost, descriptor};

    #[derive(Default)]
    struct TestPlugin {
        id: &'static str,
        priority: i32,
        dependencies: Vec<PluginDependency>,
        fail_enable: bool,
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

        fn on_enable(&mut self) -> Result<(), PluginError> {
            if self.fail_enable {
                Err(PluginError::Descriptor("enable failed".to_string()))
            } else {
                Ok(())
            }
        }

        fn contributions(&self) -> Result<ContributionSet, PluginError> {
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
        assert!(snapshot.catalog_items.iter().any(|item| {
            item.source == az_aio_plugin_api::CatalogSource::Wasm && !item.installed
        }));
    }

    #[test]
    fn symlink_skill_directories_are_scanned_by_git_skills_provider() {
        let temp = TempDir::new().expect("create temp dir");
        let real_root = temp.path().join("real-skills");
        let link_root = temp.path().join("linked-skills");
        fs::create_dir_all(real_root.join("demo")).expect("create skill dir");
        fs::write(
            real_root.join("demo/SKILL.md"),
            "---\nname: Rust Gradle Design Skill\ndescription: Rust cargo and Gradle UI design workflow\n---\n",
        )
        .expect("write skill");
        create_symlink(&real_root, &link_root);

        let scanned =
            az_aio_plugin_git_skills::scan_skill_roots(&[az_aio_plugin_git_skills::SkillRoot {
                path: link_root,
                section: "测试".to_string(),
                source: az_aio_plugin_api::CatalogSource::User,
            }]);

        assert_eq!(scanned.items[0].kind, CatalogItemKind::Skill);
        assert_eq!(scanned.items[0].name, "Rust Gradle Design Skill");
        // The skill catalog should expose inferred tags so the desktop can filter
        // and bulk-toggle visible skills without parsing SKILL.md in Dioxus.
        assert!(scanned.items[0].tags.iter().any(|tag| tag.id == "dev.rust"));
        assert!(
            scanned.items[0]
                .tags
                .iter()
                .any(|tag| tag.id == "dev.gradle")
        );
        assert!(scanned.items[0].tags.iter().any(|tag| tag.id == "design"));
    }

    #[test]
    fn builtin_shell_metadata_stays_inside_plugin_catalog() {
        let snapshot = PluginHost::with_builtin_plugins().load_snapshot();

        // Shell metadata belongs to the plugin menu grid, not the global left rail.
        assert!(
            !snapshot
                .nav_items
                .iter()
                .any(|item| { item.route == "/cli" || item.route == "/environment" })
        );
        assert!(!snapshot.shell_entries.is_empty());
    }

    #[test]
    fn builtin_snapshot_uses_feature_group_plugin_ids() {
        let snapshot = PluginHost::with_builtin_plugins().load_snapshot();
        let plugin_ids = snapshot
            .plugins
            .iter()
            .map(|plugin| plugin.descriptor.id.as_str())
            .collect::<Vec<_>>();

        assert!(plugin_ids.contains(&"settings"));
        assert!(plugin_ids.contains(&"search"));
        assert!(plugin_ids.contains(&"projects"));
        assert!(plugin_ids.contains(&"git/skills"));
        assert!(plugin_ids.contains(&"git/clis"));
        assert!(plugin_ids.contains(&"git/envs"));
        assert!(plugin_ids.contains(&"git/notes"));
        assert!(
            snapshot
                .ui_contributions
                .iter()
                .any(|item| item.id == "projects.ui.sidebar")
        );
        assert!(
            snapshot
                .ui_contributions
                .iter()
                .any(|item| item.id == "projects.ui.content")
        );
    }

    #[test]
    fn planned_feature_plugins_expose_sandbox_contracts() {
        let snapshot = PluginHost::with_builtin_plugins().load_snapshot();
        let planned_plugins = [
            "settings",
            "search",
            "projects",
            "git/skills",
            "git/clis",
            "git/envs",
            "git/notes",
        ];

        for plugin_id in planned_plugins {
            let contributions = plugin_contributions(&snapshot, plugin_id);
            assert!(
                !contributions.ui_contributions.is_empty(),
                "{plugin_id} must declare at least one UI contribution"
            );
        }

        for plugin_id in [
            "settings",
            "search",
            "projects",
            "git/skills",
            "git/clis",
            "git/envs",
        ] {
            let contributions = plugin_contributions(&snapshot, plugin_id);
            assert!(
                !contributions.backend_apis.is_empty(),
                "{plugin_id} must declare backend API debug surfaces"
            );
        }

        let projects = plugin_contributions(&snapshot, "projects");
        assert!(
            projects
                .ui_contributions
                .iter()
                .any(|item| item.slot == UiContributionSlot::ProjectSidebar)
        );
        assert!(
            projects
                .ui_contributions
                .iter()
                .any(|item| item.slot == UiContributionSlot::ProjectContent)
        );

        let settings_debug = PluginSandboxDebugReport::from_contributions(plugin_contributions(
            &snapshot, "settings",
        ));
        assert!(
            settings_debug
                .settings_defaults
                .iter()
                .any(|item| item.key == "projects.default_sync_root"
                    && item.value == "az-sync/workspace")
        );

        let notes = plugin_contributions(&snapshot, "git/notes");
        assert!(notes.backend_apis.is_empty());
    }

    fn plugin_contributions<'a>(
        snapshot: &'a super::HostSnapshot,
        plugin_id: &str,
    ) -> &'a ContributionSet {
        snapshot
            .plugin_contributions
            .iter()
            .find(|record| record.plugin_id == plugin_id)
            .map(|record| &record.contributions)
            .unwrap_or_else(|| panic!("{plugin_id} contributions should exist"))
    }

    #[cfg(unix)]
    fn create_symlink(source: &Path, link: &Path) {
        std::os::unix::fs::symlink(source, link).expect("create symlink");
    }

    #[cfg(windows)]
    fn create_symlink(source: &Path, link: &Path) {
        std::os::windows::fs::symlink_dir(source, link).expect("create symlink");
    }
}
