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
use az_derive_aliases::{apply, serde_eq, serde_kebab_code_enum, serde_kebab_eq};
use std::collections::BTreeMap;
use uuid::Uuid;

// ─── Plugin Metadata ────────────────────────────────────────────────

/// 每个 `.azplugin` 包内携带的插件清单。
///
/// 清单是插件进入宿主前的静态契约，宿主会据此完成版本、入口点、扩展点和权限检查。
#[apply(serde_eq)]
pub struct PluginManifest {
    /// 全局唯一插件标识，例如 `com.addzero.rhai-engine`。
    pub id: String,
    /// 面向用户展示的插件名称。
    pub name: String,
    /// 插件自身的 SemVer 版本。
    pub version: String,
    /// 插件能力摘要。
    pub description: String,
    /// 插件作者或组织。
    pub author: String,
    /// 插件要求的最低 AIO 平台版本。
    pub min_platform_version: String,
    /// WASM 二进制入口路径或入口符号。
    pub entry: String,
    /// 插件声明要挂载的扩展点。
    pub extension_points: Vec<ExtensionPoint>,
    /// 插件请求的宿主权限。
    pub permissions: Vec<String>,
    /// 扩展元数据键值对。
    pub metadata: BTreeMap<String, String>,
}

/// 插件可挂载到平台的扩展点。
#[apply(serde_kebab_eq)]
pub enum ExtensionPoint {
    /// 注册新的脚本引擎，例如 Rhai 或 Python。
    ScriptEngine,
    /// 注册 AI 能力，例如 LLM provider 或 prompt 模板。
    AiProvider,
    /// 注入菜单项或 UI 页面。
    UiContribution,
    /// 注册任务流节点类型。
    TaskNode,
    /// 注册 CLI 命令。
    CliCommand,
    /// 注册低代码模板生成器。
    TemplateGenerator,
    /// 自定义扩展点名称。
    Custom(String),
}

// ─── Plugin Lifecycle ───────────────────────────────────────────────

/// 插件在宿主运行时中的生命周期状态。
#[apply(serde_kebab_code_enum)]
pub enum PluginState {
    /// 插件已安装但尚未激活。
    Installed,
    /// 插件已运行并正在贡献扩展能力。
    Active,
    /// 插件已暂停，扩展能力不可用。
    Disabled,
    /// 插件进入错误状态。
    Error,
}

/// 宿主加载插件后返回的运行时句柄。
#[apply(serde_eq)]
pub struct PluginHandle {
    /// 本次加载生成的运行时实例 id。
    pub id: Uuid,
    /// 插件静态清单。
    pub manifest: PluginManifest,
    /// 插件当前生命周期状态。
    pub state: PluginState,
}

// ─── Extension Trait (the core contract) ────────────────────────────

/// 所有 WASM 插件都必须满足的宿主侧生命周期契约。
///
/// 这是宿主观察到的 trait 形态；具体 Wasmtime/WIT 绑定负责把 WASM 调用桥接到这些生命周期方法。
pub trait Plugin: Send + Sync {
    /// 插件被加载到宿主时调用。
    fn on_load(&mut self) -> anyhow::Result<()>;

    /// 插件被启用、开始贡献扩展能力时调用。
    fn on_enable(&mut self) -> anyhow::Result<()>;

    /// 插件被禁用、停止贡献扩展能力时调用。
    fn on_disable(&mut self) -> anyhow::Result<()>;

    /// 插件从宿主卸载前调用。
    fn on_unload(&mut self) -> anyhow::Result<()>;

    /// 返回插件清单。
    fn manifest(&self) -> &PluginManifest;

    /// 返回插件当前状态。
    fn state(&self) -> PluginState;
}

// ─── Plugin Registry (host-side) ────────────────────────────────────

/// 宿主侧插件注册表，负责统一管理已加载插件的生命周期。
pub trait PluginRegistry: Send + Sync {
    /// 根据插件清单和 WASM 字节加载插件。
    fn load(&self, manifest: PluginManifest, wasm_bytes: Vec<u8>) -> anyhow::Result<PluginHandle>;

    /// 按运行时 id 卸载插件。
    fn unload(&self, id: &Uuid) -> anyhow::Result<()>;

    /// 启用已加载插件。
    fn enable(&self, id: &Uuid) -> anyhow::Result<()>;

    /// 禁用已加载插件。
    fn disable(&self, id: &Uuid) -> anyhow::Result<()>;

    /// 列出当前已加载插件。
    fn list(&self) -> Vec<PluginHandle>;
}
