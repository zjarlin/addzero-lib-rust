use std::rc::Rc;
use std::sync::Arc;

use aio_plugin_api::{ExtensionPoint, PluginHandle, PluginManifest, PluginRegistry, PluginState};
use aio_runtime::RuntimePluginRegistry;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::LocalBoxFuture;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PluginDescriptorDto {
    pub runtime_id: Option<String>,
    pub manifest_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub min_platform_version: String,
    pub entry: String,
    pub extension_points: Vec<String>,
    pub permissions: Vec<String>,
    pub state: String,
    pub builtin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInstallRequestDto {
    pub manifest: PluginManifestDto,
    #[serde(default)]
    pub wasm_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifestDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub min_platform_version: String,
    pub entry: String,
    pub extension_points: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

pub trait PluginsApi: 'static {
    fn list_builtin_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>>;
    fn list_loaded_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>>;
    fn install_plugin(
        &self,
        input: PluginInstallRequestDto,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>>;
    fn enable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>>;
    fn disable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>>;
    fn uninstall_plugin(&self, runtime_id: String) -> LocalBoxFuture<'_, Result<(), String>>;
}

pub type SharedPluginsApi = Rc<dyn PluginsApi>;

pub fn default_plugins_api() -> SharedPluginsApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserPluginsApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedPluginsApi)
    }
}

static RUNTIME_REGISTRY: Lazy<Arc<RuntimePluginRegistry>> = Lazy::new(|| {
    let registry = Arc::new(RuntimePluginRegistry::new());
    for manifest in builtin_manifests() {
        let _ = registry.load(manifest, Vec::new());
    }
    registry
});

fn builtin_manifests() -> Vec<PluginManifest> {
    vec![
        PluginManifest {
            id: "com.addzero.engine.rhai".to_string(),
            name: "Rhai Script Engine".to_string(),
            version: "0.1.0".to_string(),
            description: "Built-in Rhai execution engine for script runtime and env evaluation."
                .to_string(),
            author: "addzero".to_string(),
            min_platform_version: "0.1.0".to_string(),
            entry: "builtin:rhai".to_string(),
            extension_points: vec![ExtensionPoint::ScriptEngine],
            permissions: vec!["script.run".to_string()],
            metadata: Default::default(),
        },
        PluginManifest {
            id: "com.addzero.ai.openai-chat".to_string(),
            name: "OpenAI Chat Provider".to_string(),
            version: "0.1.0".to_string(),
            description: "Built-in OpenAI compatible provider used by the platform chat layer."
                .to_string(),
            author: "addzero".to_string(),
            min_platform_version: "0.1.0".to_string(),
            entry: "builtin:openai-chat".to_string(),
            extension_points: vec![ExtensionPoint::AiProvider],
            permissions: vec!["network.outbound".to_string()],
            metadata: Default::default(),
        },
        PluginManifest {
            id: "com.addzero.ui.admin-workbench".to_string(),
            name: "Admin Workbench UI".to_string(),
            version: "0.1.0".to_string(),
            description: "Built-in workbench shell and admin UI contribution.".to_string(),
            author: "addzero".to_string(),
            min_platform_version: "0.1.0".to_string(),
            entry: "builtin:admin-workbench".to_string(),
            extension_points: vec![ExtensionPoint::UiContribution],
            permissions: vec!["ui.render".to_string()],
            metadata: Default::default(),
        },
        PluginManifest {
            id: "com.addzero.template.cli-generator".to_string(),
            name: "CLI Generator Scaffold".to_string(),
            version: "0.1.0".to_string(),
            description: "Template generator for future CLI and scaffold exports.".to_string(),
            author: "addzero".to_string(),
            min_platform_version: "0.1.0".to_string(),
            entry: "builtin:cli-generator".to_string(),
            extension_points: vec![ExtensionPoint::TemplateGenerator],
            permissions: vec!["filesystem.write".to_string()],
            metadata: Default::default(),
        },
    ]
}

fn to_dto(handle: &PluginHandle, builtin: bool) -> PluginDescriptorDto {
    PluginDescriptorDto {
        runtime_id: Some(handle.id.to_string()),
        manifest_id: handle.manifest.id.clone(),
        name: handle.manifest.name.clone(),
        version: handle.manifest.version.clone(),
        description: handle.manifest.description.clone(),
        author: handle.manifest.author.clone(),
        min_platform_version: handle.manifest.min_platform_version.clone(),
        entry: handle.manifest.entry.clone(),
        extension_points: handle
            .manifest
            .extension_points
            .iter()
            .map(extension_point_label)
            .collect(),
        permissions: handle.manifest.permissions.clone(),
        state: plugin_state_label(handle.state).to_string(),
        builtin,
    }
}

fn extension_point_label(point: &ExtensionPoint) -> String {
    match point {
        ExtensionPoint::ScriptEngine => "ScriptEngine".to_string(),
        ExtensionPoint::AiProvider => "AiProvider".to_string(),
        ExtensionPoint::UiContribution => "UiContribution".to_string(),
        ExtensionPoint::TaskNode => "TaskNode".to_string(),
        ExtensionPoint::CliCommand => "CliCommand".to_string(),
        ExtensionPoint::TemplateGenerator => "TemplateGenerator".to_string(),
        ExtensionPoint::Custom(value) => format!("Custom:{value}"),
    }
}

fn plugin_state_label(state: PluginState) -> &'static str {
    match state {
        PluginState::Installed => "Installed",
        PluginState::Active => "Active",
        PluginState::Disabled => "Disabled",
        PluginState::Error => "Error",
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserPluginsApi;

#[cfg(target_arch = "wasm32")]
impl PluginsApi for BrowserPluginsApi {
    fn list_builtin_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>> {
        Box::pin(async move { super::browser_http::get_json("/api/plugins/builtin").await })
    }

    fn list_loaded_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>> {
        Box::pin(async move { super::browser_http::get_json("/api/plugins").await })
    }

    fn install_plugin(
        &self,
        input: PluginInstallRequestDto,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(
            async move { super::browser_http::post_json("/api/plugins/install", &input).await },
        )
    }

    fn enable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/plugins/{runtime_id}/enable"),
                &serde_json::json!({}),
            )
            .await
        })
    }

    fn disable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(async move {
            super::browser_http::post_json(
                &format!("/api/plugins/{runtime_id}/disable"),
                &serde_json::json!({}),
            )
            .await
        })
    }

    fn uninstall_plugin(&self, runtime_id: String) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/plugins/{runtime_id}")).await
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedPluginsApi;

#[cfg(not(target_arch = "wasm32"))]
impl PluginsApi for EmbeddedPluginsApi {
    fn list_builtin_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>> {
        Box::pin(async move { Ok(list_builtin_plugins_on_server().await) })
    }

    fn list_loaded_plugins(&self) -> LocalBoxFuture<'_, Result<Vec<PluginDescriptorDto>, String>> {
        Box::pin(async move { Ok(list_loaded_plugins_on_server().await) })
    }

    fn install_plugin(
        &self,
        input: PluginInstallRequestDto,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(async move { install_plugin_on_server(input).await })
    }

    fn enable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(async move { enable_plugin_on_server(runtime_id).await })
    }

    fn disable_plugin(
        &self,
        runtime_id: String,
    ) -> LocalBoxFuture<'_, Result<PluginDescriptorDto, String>> {
        Box::pin(async move { disable_plugin_on_server(runtime_id).await })
    }

    fn uninstall_plugin(&self, runtime_id: String) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move { uninstall_plugin_on_server(runtime_id).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_builtin_plugins_on_server() -> Vec<PluginDescriptorDto> {
    RUNTIME_REGISTRY
        .list()
        .into_iter()
        .map(|handle| to_dto(&handle, true))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_loaded_plugins_on_server() -> Vec<PluginDescriptorDto> {
    RUNTIME_REGISTRY
        .list()
        .into_iter()
        .map(|handle| {
            let builtin = is_builtin_manifest_id(&handle.manifest.id);
            to_dto(&handle, builtin)
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn install_plugin_on_server(
    input: PluginInstallRequestDto,
) -> Result<PluginDescriptorDto, String> {
    let manifest = dto_to_manifest(input.manifest)?;
    let builtin = is_builtin_manifest_id(&manifest.id);
    let handle = RUNTIME_REGISTRY
        .load(manifest, input.wasm_bytes)
        .map_err(|err| err.to_string())?;
    Ok(to_dto(&handle, builtin))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn enable_plugin_on_server(runtime_id: String) -> Result<PluginDescriptorDto, String> {
    let id = parse_runtime_id(&runtime_id)?;
    RUNTIME_REGISTRY
        .enable(&id)
        .map_err(|err| err.to_string())?;
    find_plugin_descriptor(&runtime_id)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn disable_plugin_on_server(runtime_id: String) -> Result<PluginDescriptorDto, String> {
    let id = parse_runtime_id(&runtime_id)?;
    RUNTIME_REGISTRY
        .disable(&id)
        .map_err(|err| err.to_string())?;
    find_plugin_descriptor(&runtime_id)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn uninstall_plugin_on_server(runtime_id: String) -> Result<(), String> {
    let descriptor = find_plugin_descriptor(&runtime_id)?;
    if descriptor.builtin {
        return Err("builtin plugins cannot be uninstalled".to_string());
    }
    let id = parse_runtime_id(&runtime_id)?;
    RUNTIME_REGISTRY.unload(&id).map_err(|err| err.to_string())
}

fn dto_to_manifest(input: PluginManifestDto) -> Result<PluginManifest, String> {
    if input.id.trim().is_empty()
        || input.name.trim().is_empty()
        || input.version.trim().is_empty()
        || input.entry.trim().is_empty()
    {
        return Err("plugin manifest fields id/name/version/entry are required".to_string());
    }

    Ok(PluginManifest {
        id: input.id.trim().to_string(),
        name: input.name.trim().to_string(),
        version: input.version.trim().to_string(),
        description: input.description.trim().to_string(),
        author: input.author.trim().to_string(),
        min_platform_version: input.min_platform_version.trim().to_string(),
        entry: input.entry.trim().to_string(),
        extension_points: input
            .extension_points
            .into_iter()
            .map(|value| extension_point_from_str(&value))
            .collect::<Result<Vec<_>, _>>()?,
        permissions: input
            .permissions
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect(),
        metadata: Default::default(),
    })
}

fn extension_point_from_str(value: &str) -> Result<ExtensionPoint, String> {
    match value.trim() {
        "ScriptEngine" => Ok(ExtensionPoint::ScriptEngine),
        "AiProvider" => Ok(ExtensionPoint::AiProvider),
        "UiContribution" => Ok(ExtensionPoint::UiContribution),
        "TaskNode" => Ok(ExtensionPoint::TaskNode),
        "CliCommand" => Ok(ExtensionPoint::CliCommand),
        "TemplateGenerator" => Ok(ExtensionPoint::TemplateGenerator),
        other if other.starts_with("Custom:") => Ok(ExtensionPoint::Custom(
            other.trim_start_matches("Custom:").to_string(),
        )),
        other => Err(format!("unsupported extension point: {other}")),
    }
}

fn is_builtin_manifest_id(manifest_id: &str) -> bool {
    builtin_manifests()
        .iter()
        .any(|item| item.id == manifest_id)
}

fn parse_runtime_id(runtime_id: &str) -> Result<uuid::Uuid, String> {
    uuid::Uuid::parse_str(runtime_id).map_err(|err| format!("invalid runtime id: {err}"))
}

fn find_plugin_descriptor(runtime_id: &str) -> Result<PluginDescriptorDto, String> {
    RUNTIME_REGISTRY
        .list()
        .into_iter()
        .find(|handle| handle.id.to_string() == runtime_id)
        .map(|handle| {
            let builtin = is_builtin_manifest_id(&handle.manifest.id);
            to_dto(&handle, builtin)
        })
        .ok_or_else(|| "plugin not found".to_string())
}
