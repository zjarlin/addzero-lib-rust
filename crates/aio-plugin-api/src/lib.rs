//! AIO 插件公共 API — Rust Trait + 类型定义
//!
//! 定义所有 WASM 插件必须实现的契约接口。
//! Wasmtime 宿主和插件通过此 crate 共享类型和方法签名。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

// ─── Plugin Metadata ────────────────────────────────────────────────

/// Plugin manifest — shipped inside every `.aio-plugin` bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, thiserror::Error)]
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
