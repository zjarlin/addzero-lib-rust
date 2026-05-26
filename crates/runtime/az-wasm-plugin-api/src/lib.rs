//! WASM 插件系统的宿主端契约类型与 trait 定义。
//!
//! 本 crate 定义了 AddZero/AIO 平台 WASM 插件系统的核心抽象，不包含任何具体实现，
//! 供宿主运行时（如 `az-wasm-plugin-host`）和插件 crate 共同依赖。
//!
//! ## 核心类型
//!
//! - [`PluginManifest`]：插件清单，描述 id、名称、版本、入口点、扩展点、权限等元数据
//! - [`ExtensionPoint`]：扩展点枚举，定义插件可挂载到平台的位置（脚本引擎、AI 服务、UI、任务节点、CLI、模板等）
//! - [`PluginState`]：插件生命周期状态（已安装 → 激活 → 禁用 / 错误）
//! - [`PluginHandle`]：运行时句柄，将 UUID 与清单和状态绑定
//!
//! ## 核心 trait
//!
//! - [`Plugin`]：所有 WASM 插件必须实现的契约，包含生命周期钩子（`on_load`、`on_enable`、`on_disable`、`on_unload`）
//! - [`PluginRegistry`]：宿主端插件管理器接口，负责加载、卸载、启用、禁用和列举插件
//!
//! ## 错误类型
//!
//! - [`PluginError`]：覆盖未找到、重复加载、权限拒绝、WASM 运行时错误等场景

use az_derive_aliases::{apply, error_eq, serde_eq, serde_kebab_code_enum, serde_kebab_eq};
use std::collections::BTreeMap;
use uuid::Uuid;

// ─── Plugin Metadata ────────────────────────────────────────────────

/// Plugin manifest shipped inside every `.azplugin` bundle.
#[apply(serde_eq)]
pub struct PluginManifest {
    /// Unique plugin identifier (e.g. "com.addzero.rhai-engine").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semver version.
    pub version: String,
    /// Short description.
    pub description: String,
    /// Plugin author/organization.
    pub author: String,
    /// Minimum AIO platform version required.
    pub min_platform_version: String,
    /// WASM binary entry point.
    pub entry: String,
    /// Extension points this plugin contributes to.
    pub extension_points: Vec<ExtensionPoint>,
    /// Permissions requested by this plugin.
    pub permissions: Vec<String>,
    /// Arbitrary metadata key-value pairs.
    pub metadata: BTreeMap<String, String>,
}

/// Where this plugin hooks into the platform.
#[apply(serde_kebab_eq)]
pub enum ExtensionPoint {
    /// Adds a new script engine (Rhai, Python, etc.).
    ScriptEngine,
    /// Adds AI capabilities (LLM provider, prompt template).
    AiProvider,
    /// Adds menu items or UI pages.
    UiContribution,
    /// Adds task flow node types.
    TaskNode,
    /// Adds CLI command.
    CliCommand,
    /// Adds a low-code template generator.
    TemplateGenerator,
    /// Custom extension point (free-form string).
    Custom(String),
}

// ─── Plugin Lifecycle ───────────────────────────────────────────────

/// State of a plugin within the runtime.
#[apply(serde_kebab_code_enum)]
pub enum PluginState {
    /// Plugin is installed but not active.
    Installed,
    /// Plugin is running and contributing extensions.
    Active,
    /// Plugin has been paused (extensions unavailable).
    Disabled,
    /// Plugin encountered an error.
    Error,
}

/// Runtime handle for a loaded plugin.
#[apply(serde_eq)]
pub struct PluginHandle {
    pub id: Uuid,
    pub manifest: PluginManifest,
    pub state: PluginState,
}

// ─── Extension Trait (the core contract) ────────────────────────────

/// Every WASM plugin must implement this trait.
/// This is the **host-side view** — the Wasmtime runtime calls these
/// methods through the WIT interface.
pub trait Plugin: Send + Sync {
    /// Called when the plugin is loaded.
    fn on_load(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is activated (start contributing extensions).
    fn on_enable(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is deactivated.
    fn on_disable(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is unloaded.
    fn on_unload(&mut self) -> Result<(), PluginError>;

    /// Return the plugin's manifest.
    fn manifest(&self) -> &PluginManifest;

    /// Return the current state.
    fn state(&self) -> PluginState;
}

// ─── Plugin Registry (host-side) ────────────────────────────────────

/// The plugin registry manages the lifecycle of all loaded plugins.
pub trait PluginRegistry: Send + Sync {
    /// Load a plugin from its WASM binary and manifest.
    fn load(
        &self,
        manifest: PluginManifest,
        wasm_bytes: Vec<u8>,
    ) -> Result<PluginHandle, PluginError>;

    /// Unload a plugin by its runtime id.
    fn unload(&self, id: &Uuid) -> Result<(), PluginError>;

    /// Enable a loaded plugin.
    fn enable(&self, id: &Uuid) -> Result<(), PluginError>;

    /// Disable a loaded plugin.
    fn disable(&self, id: &Uuid) -> Result<(), PluginError>;

    /// List all loaded plugins.
    fn list(&self) -> Vec<PluginHandle>;
}

// ─── Error ──────────────────────────────────────────────────────────

#[apply(error_eq)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),

    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("unsupported extension point: {0}")]
    UnsupportedExtension(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("WASM error: {0}")]
    Wasm(String),

    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::{ExtensionPoint, PluginState};

    #[test]
    fn plugin_state_codes_follow_manifest_values() {
        assert_eq!(PluginState::Installed.code(), "installed");
        assert_eq!(PluginState::from_code("active"), Some(PluginState::Active));
    }

    #[test]
    fn extension_points_keep_manifest_wire_values() {
        assert_eq!(
            serde_json::to_string(&ExtensionPoint::ScriptEngine).expect("serialize"),
            r#""script-engine""#
        );
        assert_eq!(
            serde_json::from_str::<ExtensionPoint>(r#""ui-contribution""#).expect("deserialize"),
            ExtensionPoint::UiContribution
        );
    }
}
