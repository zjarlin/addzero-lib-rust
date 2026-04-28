#![forbid(unsafe_code)]

use addzero_remote_model::{
    DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability,
};
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

pub type HostResult<T> = Result<T, HostError>;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("platform adapter is unavailable: {0}")]
    Unavailable(String),
}

pub trait HostPlatformAdapter {
    fn descriptor(&self, device_name: &str) -> HostResult<DeviceDescriptor>;
    fn permission_hint(&self) -> &'static str;
}

#[derive(Debug, Default)]
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
