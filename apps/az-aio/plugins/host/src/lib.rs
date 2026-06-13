#![forbid(unsafe_code)]

automod::dir!("src");

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    thread,
};

use az_aio_plugin_api::{
    AzAioPlugin, BackendApiContribution, CatalogItemContribution, CatalogItemKind, CatalogSource,
    ContributionSet, GeneratedFileContribution, NavItemContribution, PageContribution,
    PluginActivation, PluginBundleArtifactKind, PluginBundleManifest, PluginDependency,
    PluginDescriptor, PluginError, PluginKind, PluginState, SettingsSectionContribution,
    ShellEntryContribution, ToolbarActionContribution, UiContribution,
};
#[cfg(not(target_arch = "wasm32"))]
use az_aio_plugin_api::{
    NativeAzAioPlugin, NativePluginContext, NativePluginRegistration, NativePluginRuntime,
    NativeRenderContext, NativeRenderFn, NativeUiRenderer,
};
use serde::{Deserialize, Serialize};

pub use wasm_component::WasmComponentPlugin;

const PLUGIN_STATE_FILE: &str = "plugin-state.json";
const PLUGIN_MANIFEST_FILE: &str = "az-plugin.json";

#[cfg(not(target_arch = "wasm32"))]
pub fn load_az_aio_native_snapshot() -> HostSnapshot {
    let enablement = load_plugin_enablement();
    NativePluginHost::from_inventory(NativePluginContext::default())
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("az-aio native plugin server failed: {error}");
        }
    });
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
        load_native_plugins(self.plugins, self.context, enablement)
    }
}

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

    pub fn with_wasm_components_from_dir(mut self, dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        let Ok(entries) = fs::read_dir(dir) else {
            return self;
        };

        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_wasm_component_file(path))
            .collect::<Vec<_>>();
        paths.sort();

        for (path, result) in load_wasm_components(paths) {
            match result {
                Ok(plugin) => self.plugins.push(Box::new(plugin)),
                Err(error) => self.records.push(failed_record(
                    failed_wasm_descriptor(&path),
                    error.to_string(),
                )),
            }
        }

        self
    }

    pub fn with_packaged_plugins_from_dir(mut self, dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        let mut manifests = Vec::new();
        collect_plugin_manifests(dir, &mut manifests);
        manifests.sort();

        for (_, result) in load_packaged_plugins(manifests) {
            match result {
                Ok(plugin) => self.plugins.push(Box::new(plugin)),
                Err((descriptor, error)) => self.records.push(failed_record(descriptor, error)),
            }
        }

        self
    }

    pub fn load_snapshot(self) -> HostSnapshot {
        HostLoader::new(self.plugins, self.records, BTreeSet::new()).load()
    }

    pub fn load_snapshot_with_enablement(self, enablement: &PluginEnablementStore) -> HostSnapshot {
        HostLoader::new(
            self.plugins,
            self.records,
            enablement.disabled_plugin_ids.clone(),
        )
        .load()
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_az_aio_plugin_snapshot() -> HostSnapshot {
    let enablement = load_plugin_enablement();
    default_plugin_host().load_snapshot_with_enablement(&enablement)
}

pub fn default_plugin_host() -> PluginHost {
    default_plugin_package_dirs()
        .into_iter()
        .fold(PluginHost::new(), |host, dir| {
            host.with_packaged_plugins_from_dir(dir)
        })
}

pub fn default_plugin_package_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![repo_packaged_plugins_dir()];
    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("addzero").join("az-aio").join("plugins"));
    }
    dirs
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
        .unwrap_or_else(|| home_dir().join(".config"))
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

struct PluginSlot {
    plugin: Box<dyn AzAioPlugin>,
    descriptor: PluginDescriptor,
}

struct SlotActivationResult {
    record: PluginRuntimeRecord,
    plugin_contribution: Option<PluginContributionRecord>,
}

struct HostLoader {
    slots: Vec<PluginSlot>,
    records: Vec<PluginRuntimeRecord>,
    disabled_plugin_ids: BTreeSet<String>,
    contributions: ContributionSet,
    plugin_contributions: Vec<PluginContributionRecord>,
}

impl HostLoader {
    fn new(
        plugins: Vec<Box<dyn AzAioPlugin>>,
        records: Vec<PluginRuntimeRecord>,
        disabled_plugin_ids: BTreeSet<String>,
    ) -> Self {
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
            disabled_plugin_ids,
            contributions: ContributionSet::default(),
            plugin_contributions: Vec::new(),
        }
    }

    fn load(mut self) -> HostSnapshot {
        let disabled_indexes = self.disabled_slot_indexes();
        for index in &disabled_indexes {
            self.records
                .push(disabled_record(self.slots[*index].descriptor.clone()));
        }

        let (layers, failures) = activation_layers(&self.slots, &disabled_indexes);
        for (index, error) in failures {
            self.records.push(failed_record(
                self.slots[index].descriptor.clone(),
                error.to_string(),
            ));
        }

        let mut loaded_dependency_ids = HashSet::new();
        let mut slots = std::mem::take(&mut self.slots)
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>();

        for layer in layers {
            let ready_indexes = self.ready_layer_indexes(&slots, &layer, &loaded_dependency_ids);
            for result in activate_layer(&mut slots, ready_indexes) {
                let loaded = matches!(
                    result.record.state,
                    PluginState::Active | PluginState::Loaded
                );
                let plugin_id = result.record.descriptor.id.clone();
                self.ingest_activation_result(result);
                if loaded {
                    loaded_dependency_ids.insert(plugin_id);
                }
            }
        }

        self.snapshot()
    }

    fn disabled_slot_indexes(&self) -> BTreeSet<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                self.disabled_plugin_ids
                    .contains(&slot.descriptor.id)
                    .then_some(index)
            })
            .collect()
    }

    fn ready_layer_indexes(
        &mut self,
        slots: &[Option<PluginSlot>],
        layer: &[usize],
        loaded_dependency_ids: &HashSet<String>,
    ) -> Vec<usize> {
        let mut ready_indexes = Vec::new();

        for index in layer {
            let Some(slot) = slots[*index].as_ref() else {
                continue;
            };
            if self
                .records
                .iter()
                .any(|record| record.descriptor.id == slot.descriptor.id)
            {
                continue;
            }

            if let Some(dependency) = first_unloaded_required_dependency(
                &slot.descriptor.dependencies,
                loaded_dependency_ids,
            ) {
                self.records.push(failed_record(
                    slot.descriptor.clone(),
                    PluginError::DependencyFailed {
                        plugin: slot.descriptor.id.clone(),
                        dependency,
                    }
                    .to_string(),
                ));
                continue;
            }

            ready_indexes.push(*index);
        }

        ready_indexes
    }

    fn ingest_activation_result(&mut self, result: SlotActivationResult) {
        if let Some(plugin_contribution) = result.plugin_contribution {
            self.contributions
                .merge(plugin_contribution.contributions.clone());
            self.plugin_contributions.push(plugin_contribution);
        }
        self.records.push(result.record);
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
            #[cfg(not(target_arch = "wasm32"))]
            native_renderers: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            native_router: axum::Router::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native_plugins(
    plugins: Vec<Box<dyn NativeAzAioPlugin>>,
    context: NativePluginContext,
    enablement: &PluginEnablementStore,
) -> HostSnapshot {
    let mut snapshot = HostSnapshot::default();
    let mut seen_ids = HashSet::new();
    let mut seen_routes = HashSet::new();

    for plugin in plugins {
        let descriptor = plugin.descriptor();
        if !seen_ids.insert(descriptor.id.clone()) {
            snapshot.plugins.push(failed_record(
                descriptor.clone(),
                PluginError::DuplicateId(descriptor.id.clone()).to_string(),
            ));
            continue;
        }
        if !enablement.plugin_enabled(&descriptor.id) {
            snapshot.plugins.push(disabled_record(descriptor));
            continue;
        }

        match load_native_plugin(plugin.as_ref(), descriptor.clone(), context.clone()) {
            Ok(loaded) => {
                if let Some((method, path)) = first_duplicate_backend_route(
                    &loaded.contributions.backend_apis,
                    &mut seen_routes,
                ) {
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
                        contributions: loaded.contributions.clone(),
                    });
                merge_snapshot_contributions(&mut snapshot, loaded.contributions);
                snapshot.native_renderers.extend(loaded.runtime.renderers);
                snapshot.native_router = snapshot.native_router.merge(loaded.runtime.router);
                snapshot.plugins.push(PluginRuntimeRecord {
                    descriptor,
                    state: PluginState::Active,
                    error: None,
                });
            }
            Err(error) => {
                snapshot
                    .plugins
                    .push(failed_record(descriptor, error.to_string()));
            }
        }
    }

    sort_snapshot(&mut snapshot);
    snapshot
}

#[cfg(not(target_arch = "wasm32"))]
struct LoadedNativePlugin {
    contributions: ContributionSet,
    runtime: NativePluginRuntime,
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native_plugin(
    plugin: &dyn NativeAzAioPlugin,
    descriptor: PluginDescriptor,
    context: NativePluginContext,
) -> anyhow::Result<LoadedNativePlugin> {
    let contributions = plugin.contributions().map_err(|error| {
        anyhow::anyhow!(lifecycle_message(&descriptor.id, "native-contributions", error))
    })?;
    let runtime = plugin.runtime(context).map_err(|error| {
        anyhow::anyhow!(
            "插件 `{}` native runtime 阶段失败：{}",
            descriptor.id,
            error
        )
    })?;
    if let Some(startup) = runtime.startup {
        startup(NativePluginContext::default()).map_err(|error| {
            anyhow::anyhow!("插件 `{}` native startup 阶段失败：{}", descriptor.id, error)
        })?;
    }
    Ok(LoadedNativePlugin {
        contributions,
        runtime,
    })
}

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
    snapshot.ui_contributions.extend(contributions.ui_contributions);
    snapshot.backend_apis.extend(contributions.backend_apis);
    snapshot.toolbar_actions.extend(contributions.toolbar_actions);
    snapshot.settings_sections.extend(contributions.settings_sections);
    snapshot.shell_entries.extend(contributions.shell_entries);
    snapshot.generated_files.extend(contributions.generated_files);
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
    snapshot.catalog_items.extend(plugin_catalog_items(&snapshot.plugins));
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

fn collect_plugin_manifests(dir: &Path, manifests: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifests(&path, manifests);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == PLUGIN_MANIFEST_FILE)
        {
            manifests.push(path);
        }
    }
}

fn load_packaged_plugin(
    manifest_path: &Path,
) -> Result<WasmComponentPlugin, (PluginDescriptor, String)> {
    let manifest = read_plugin_bundle_manifest(manifest_path)?;
    let Some(component_path) = wasm_component_artifact_path(manifest_path, &manifest) else {
        return Err((
            manifest.descriptor,
            PluginError::Wasm {
                plugin: manifest.bundle_id,
                message: "package does not declare a wasm component artifact".to_string(),
            }
            .to_string(),
        ));
    };

    WasmComponentPlugin::from_file(&component_path)
        .map_err(|error| (manifest.descriptor, error.to_string()))
}

fn load_packaged_plugins(
    manifests: Vec<PathBuf>,
) -> Vec<(
    PathBuf,
    Result<WasmComponentPlugin, (PluginDescriptor, String)>,
)> {
    thread::scope(|scope| {
        manifests
            .into_iter()
            .map(|manifest_path| {
                scope.spawn(move || {
                    let result = load_packaged_plugin(&manifest_path);
                    (manifest_path, result)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("plugin package loader panicked"))
            .collect()
    })
}

fn load_wasm_components(
    paths: Vec<PathBuf>,
) -> Vec<(PathBuf, Result<WasmComponentPlugin, PluginError>)> {
    thread::scope(|scope| {
        paths
            .into_iter()
            .map(|path| {
                scope.spawn(move || {
                    let result = WasmComponentPlugin::from_file(&path);
                    (path, result)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("wasm component loader panicked"))
            .collect()
    })
}

fn read_plugin_bundle_manifest(
    manifest_path: &Path,
) -> Result<PluginBundleManifest, (PluginDescriptor, String)> {
    let fallback = failed_package_descriptor(manifest_path);
    let contents = fs::read_to_string(manifest_path)
        .map_err(|error| (fallback.clone(), package_io_error(manifest_path, error)))?;
    serde_json::from_str(&contents).map_err(|error| {
        (
            fallback,
            PluginError::Descriptor(format!(
                "package manifest `{}` is invalid: {error}",
                manifest_path.display()
            ))
            .to_string(),
        )
    })
}

fn wasm_component_artifact_path(
    manifest_path: &Path,
    manifest: &PluginBundleManifest,
) -> Option<PathBuf> {
    manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == PluginBundleArtifactKind::WasmComponent)
        .and_then(|artifact| artifact.path.as_deref())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                manifest_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(path)
            }
        })
}

fn disabled_record(descriptor: PluginDescriptor) -> PluginRuntimeRecord {
    PluginRuntimeRecord {
        descriptor,
        state: PluginState::Disabled,
        error: None,
    }
}

fn is_wasm_component_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wasm"))
}

fn failed_package_descriptor(path: &Path) -> PluginDescriptor {
    PluginDescriptor {
        id: format!("package/{}", sanitize_id(&path.display().to_string())),
        name: "插件包加载失败".to_string(),
        version: "未知".to_string(),
        description: "插件包 manifest 读取或解析失败。".to_string(),
        activation: PluginActivation::Eager,
        priority: 0,
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        permissions: vec![format!("读取 {}", path.display())],
        kind: PluginKind::WasmComponent,
    }
}

fn package_io_error(path: &Path, error: io::Error) -> String {
    PluginError::Io {
        plugin: path.display().to_string(),
        path: path.display().to_string(),
        message: error.to_string(),
    }
    .to_string()
}

fn failed_wasm_descriptor(path: &Path) -> PluginDescriptor {
    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Wasm 组件加载失败")
        .to_string();

    PluginDescriptor {
        id: format!("wasm/{}", sanitize_id(&path.display().to_string())),
        name,
        version: "未知".to_string(),
        description: "Wasm 组件描述符发现失败。".to_string(),
        activation: PluginActivation::Eager,
        priority: 0,
        dependencies: Vec::new(),
        capabilities: Vec::new(),
        permissions: vec![format!("读取 {}", path.display())],
        kind: PluginKind::WasmComponent,
    }
}

fn activation_layers(
    slots: &[PluginSlot],
    disabled_indexes: &BTreeSet<usize>,
) -> (Vec<Vec<usize>>, Vec<(usize, PluginError)>) {
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
            match id_to_index.get(&dependency.id) {
                Some(dependency_index) if disabled_indexes.contains(dependency_index) => {
                    failed.insert(index);
                    failures.push((
                        index,
                        PluginError::MissingDependency {
                            plugin: slot.descriptor.id.clone(),
                            dependency: dependency.id.clone(),
                        },
                    ));
                }
                Some(_) => {}
                None => {
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
    }

    let mut remaining = slots
        .iter()
        .enumerate()
        .filter_map(|(index, _)| {
            (!failed.contains(&index) && !disabled_indexes.contains(&index)).then_some(index)
        })
        .collect::<BTreeSet<_>>();
    let mut activated_ids = HashSet::new();
    let mut layers = Vec::new();

    while !remaining.is_empty() {
        let mut ready = remaining
            .iter()
            .copied()
            .filter(|index| {
                dependency_order_ready(
                    &slots[*index].descriptor,
                    &id_to_index,
                    disabled_indexes,
                    &activated_ids,
                )
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

        let mut layer = Vec::new();
        for index in ready {
            remaining.remove(&index);
            activated_ids.insert(slots[index].descriptor.id.clone());
            layer.push(index);
        }
        layers.push(layer);
    }

    (layers, failures)
}

fn activate_layer(
    slots: &mut [Option<PluginSlot>],
    indexes: Vec<usize>,
) -> Vec<SlotActivationResult> {
    let mut indexed_results = thread::scope(|scope| {
        indexes
            .into_iter()
            .enumerate()
            .filter_map(|(order, index)| slots[index].take().map(|slot| (order, slot)))
            .map(|(order, slot)| scope.spawn(move || (order, activate_slot(slot))))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("plugin activation worker panicked"))
            .collect::<Vec<_>>()
    });

    indexed_results.sort_by_key(|(index, _)| *index);
    indexed_results
        .into_iter()
        .map(|(_, result)| result)
        .collect()
}

fn activate_slot(mut slot: PluginSlot) -> SlotActivationResult {
    let descriptor = slot.descriptor.clone();
    if let Err(error) = slot.plugin.on_load() {
        return SlotActivationResult {
            record: failed_record(
                descriptor.clone(),
                lifecycle_message(&descriptor.id, "on-load", error),
            ),
            plugin_contribution: None,
        };
    }

    if descriptor.activation == PluginActivation::Eager {
        if let Err(error) = slot.plugin.on_enable() {
            return SlotActivationResult {
                record: failed_record(
                    descriptor.clone(),
                    lifecycle_message(&descriptor.id, "on-enable", error),
                ),
                plugin_contribution: None,
            };
        }
    }

    match slot.plugin.contributions() {
        Ok(contributions) => {
            let state = if descriptor.activation == PluginActivation::Eager {
                PluginState::Active
            } else {
                PluginState::Loaded
            };
            SlotActivationResult {
                record: PluginRuntimeRecord {
                    descriptor: descriptor.clone(),
                    state,
                    error: None,
                },
                plugin_contribution: Some(PluginContributionRecord {
                    plugin_id: descriptor.id,
                    contributions,
                }),
            }
        }
        Err(error) => SlotActivationResult {
            record: failed_record(
                descriptor.clone(),
                lifecycle_message(&descriptor.id, "contributions", error),
            ),
            plugin_contribution: None,
        },
    }
}

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

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn repo_packaged_plugins_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(|root| root.join("target").join("az-platform").join("plugins"))
        .unwrap_or_else(|| PathBuf::from("target/az-platform/plugins"))
}

fn dependency_order_ready(
    descriptor: &PluginDescriptor,
    id_to_index: &HashMap<String, usize>,
    disabled_indexes: &BTreeSet<usize>,
    activated_ids: &HashSet<String>,
) -> bool {
    descriptor.dependencies.iter().all(|dependency| {
        id_to_index
            .get(&dependency.id)
            .is_none_or(|dependency_index| {
                disabled_indexes.contains(dependency_index)
                    || activated_ids.contains(&dependency.id)
            })
    })
}

fn first_unloaded_required_dependency(
    dependencies: &[PluginDependency],
    loaded_dependency_ids: &HashSet<String>,
) -> Option<String> {
    dependencies
        .iter()
        .filter(|dependency| !dependency.optional)
        .find(|dependency| !loaded_dependency_ids.contains(&dependency.id))
        .map(|dependency| dependency.id.clone())
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
            source: CatalogSource::Wasm,
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
        kind: PluginKind::WasmComponent,
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
    use std::{
        fs,
        time::{Duration, Instant},
    };

    use az_aio_plugin_api::{
        BackendApiContribution, CatalogItemKind, ContributionSet, NavItemContribution,
        PluginDependency, PluginError, PluginState, UiContribution, UiContributionSlot,
    };
    use tempfile::TempDir;

    use super::{
        AzAioPlugin, PluginEnablementStore, PluginHost, descriptor, set_plugin_enabled_at,
    };

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

        fn on_enable(&mut self) -> Result<(), PluginError> {
            if !self.enable_delay.is_zero() {
                std::thread::sleep(self.enable_delay);
            }
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

        assert!(snapshot.plugins.iter().any(|plugin| {
            plugin.descriptor.id == "sync" && plugin.state == PluginState::Active
        }));
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
        assert!(snapshot.catalog_items.iter().any(|item| {
            item.source == az_aio_plugin_api::CatalogSource::Wasm && !item.installed
        }));
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
        let store: PluginEnablementStore =
            serde_json::from_str(&stored).expect("parse plugin state");
        assert!(store.disabled_plugin_ids.contains("settings"));

        set_plugin_enabled_at(&path, "settings", true).expect("enable plugin");
        let stored = fs::read_to_string(&path).expect("read plugin state");
        let store: PluginEnablementStore =
            serde_json::from_str(&stored).expect("parse plugin state");
        assert!(!store.disabled_plugin_ids.contains("settings"));
    }
}
