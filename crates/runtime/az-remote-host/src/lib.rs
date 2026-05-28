//! # az-remote-host
//!
//! 远程控制宿主端的平台抽象层，负责检测当前操作系统平台、生成设备描述符并提供平台特定的权限提示。
//!
//! ## 核心职责
//!
//! - [`HostPlatformAdapter`] trait：为不同操作系统实现宿主端设备描述和权限提示的统一接口。
//! - [`MockHostPlatformAdapter`]：开箱即用的默认实现，可直接用于测试或快速集成。
//! - [`current_platform`]：运行时平台检测函数，支持 macOS、Windows、Linux (X11/Wayland) 及浏览器环境。
//! - [`HostError`] / [`HostResult`]：统一的错误类型与结果别名。
//!
//! ## 设备描述
//!
//! 通过 [`az_remote_model::DeviceDescriptor`] 填充宿主设备的唯一标识、平台类型、角色（Host）、
//! 在线状态和完整会话能力，供远控信令层直接使用。
//!
//! ## 平台感知
//!
//! 在 Linux 环境下，会读取 `XDG_SESSION_TYPE` 环境变量区分 Wayland 与 X11；
//! Wayland 首版仅保证受限兼容，[`MockHostPlatformAdapter`] 会自动附加相应备注和权限提示。

#![forbid(unsafe_code)]

use az_derive_aliases::{apply, error_eq, plain_default_copy_eq};
use az_remote_model::{
    DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability,
};
use chrono::Utc;
use uuid::Uuid;

/// 远程宿主平台适配层统一返回类型。
pub type HostResult<T> = Result<T, HostError>;

/// 宿主平台适配错误。
#[apply(error_eq)]
pub enum HostError {
    /// 当前平台适配器不可用。
    #[error("platform adapter is unavailable: {0}")]
    Unavailable(String),
}

/// 远程宿主端平台适配器接口。
///
/// 不同操作系统可通过该 trait 注入设备描述和权限提示，信令层只依赖统一契约。
pub trait HostPlatformAdapter {
    /// 生成当前宿主设备描述。
    fn descriptor(&self, device_name: &str) -> HostResult<DeviceDescriptor>;
    /// 返回当前平台需要用户授予的权限提示。
    fn permission_hint(&self) -> &'static str;
}

/// 默认宿主平台适配器。
///
/// 该实现只做平台检测和设备描述组装，不执行真实屏幕捕获或输入控制。
#[apply(plain_default_copy_eq)]
pub struct MockHostPlatformAdapter;

impl HostPlatformAdapter for MockHostPlatformAdapter {
    fn descriptor(&self, device_name: &str) -> HostResult<DeviceDescriptor> {
        Ok(DeviceDescriptor {
            device_id: Uuid::new_v4(),
            device_name: device_name.into(),
            platform: current_platform(),
            role: DeviceRole::Host,
            capabilities: SessionCapability::full_host(),
            online_status: OnlineStatus::Online,
            last_seen_at: Utc::now(),
            notes: platform_note(),
        })
    }

    fn permission_hint(&self) -> &'static str {
        match current_platform() {
            RemotePlatform::MacOs => "需要屏幕录制和辅助功能权限。",
            RemotePlatform::Windows => "需要桌面捕获和输入模拟权限。",
            RemotePlatform::LinuxWayland => "Wayland 仅保证受限兼容，建议优先 X11。",
            RemotePlatform::LinuxX11 => "X11 首版支持远控，Wayland 视桌面环境而定。",
            RemotePlatform::Browser => "浏览器不是 host 目标。",
        }
    }
}

/// 检测当前编译目标和运行会话对应的远控平台。
#[must_use]
pub fn current_platform() -> RemotePlatform {
    if cfg!(target_os = "macos") {
        RemotePlatform::MacOs
    } else if cfg!(target_os = "windows") {
        RemotePlatform::Windows
    } else if cfg!(target_os = "linux") {
        match std::env::var("XDG_SESSION_TYPE").ok().as_deref() {
            Some("wayland") => RemotePlatform::LinuxWayland,
            _ => RemotePlatform::LinuxX11,
        }
    } else {
        RemotePlatform::Browser
    }
}

fn platform_note() -> Option<String> {
    match current_platform() {
        RemotePlatform::LinuxWayland => Some("Wayland 首版不承诺完整输入控制能力。".into()),
        _ => None,
    }
}
