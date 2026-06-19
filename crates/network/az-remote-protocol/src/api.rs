//! 远程桌面控制协议，基于 JSON 序列化的控制帧定义。
//!
//! # 核心类型
//!
//! - [`ControlFrame`] — 控制帧枚举，覆盖远程桌面交互的全部消息类型：
//!   - `Hello` — 设备握手（携带 [`DeviceDescriptor`] 和中继令牌）
//!   - `DeviceSnapshot` — 设备列表快照
//!   - `SessionOffer` / `SessionAccept` — 会话协商
//!   - `PermissionGrant` — 权限授予结果
//!   - `ClipboardSync` — 剪贴板同步
//!   - `FileChunk` / `VideoChunk` — 文件和视频数据块传输
//!   - `InputEvent` — 远程输入事件
//!   - `Heartbeat` — 心跳保活
//!   - `Error` — 错误码和消息
//! - [`StreamKind`] — 流类型枚举（Control / Video / Input / Clipboard / File），
//!   用于多路复用时标识数据通道类型。
//!
//! # 关键功能
//!
//! - **JSON 序列化**：所有协议消息通过 `serde_json` 进行二进制序列化/反序列化，
//!   `ControlFrame::to_json_bytes()` 和 `ControlFrame::from_json_bytes()` 提供便捷的转换方法。
//! - **安全脱敏**：`DeviceHello` 的 `relay_token` 在 `Debug` 输出中被省略。
//! - **禁止 unsafe**：整个 crate 使用 `#![forbid(unsafe_code)]`。
//! - **共享模型**：设备描述、会话请求/授权、输入事件、剪贴板/文件/视频信封等
//!   数据类型由 `az-remote-model` crate 统一提供，本 crate 专注于协议帧的组装与解析。
//!
//! # 快速开始
//!
//! ```rust
//! use az_remote_protocol::api::{ControlFrame, DeviceHello, StreamKind};
//! use az_remote_model::api::{DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability};
//! use chrono::Utc;
//! use uuid::Uuid;
//!
//! # fn main() -> anyhow::Result<()> {
//! let hello = DeviceHello {
//!     device: DeviceDescriptor {
//!         device_id: Uuid::new_v4(),
//!         device_name: "我的电脑".into(),
//!         platform: RemotePlatform::LinuxX11,
//!         role: DeviceRole::Host,
//!         capabilities: SessionCapability::full_host(),
//!         online_status: OnlineStatus::Online,
//!         last_seen_at: Utc::now(),
//!         notes: None,
//!     },
//!     relay_token: "secret-token".into(),
//! };
//!
//! let frame = ControlFrame::Hello(hello);
//! let bytes = frame.to_json_bytes()?;
//! let restored = ControlFrame::from_json_bytes(&bytes)?;
//! assert_eq!(frame, restored);
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use az_remote_model::api::{
    ClipboardPayload, DeviceDescriptor, FileTransferEnvelope, RemoteInputEvent, SessionGrant,
    SessionRequest, VideoFrameEnvelope,
};

/// 多路复用通道类型。
///
/// 该枚举的 code 字符串用于中继层标识数据通道，不用于区分 [`ControlFrame`] 的具体变体。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum StreamKind {
    Control,
    Video,
    Input,
    Clipboard,
    File,
}

impl StreamKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 设备握手帧。
///
/// `relay_token` 是一次性中继凭证，`Debug` 输出必须保持脱敏。
#[derive(Clone, derive_more::Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceHello {
    pub device: DeviceDescriptor,
    #[debug(skip)]
    pub relay_token: String,
}

/// 请求建立远程会话的控制帧负载。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionOffer {
    pub request: SessionRequest,
}

/// 接受远程会话后的授权结果负载。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionAccept {
    pub grant: SessionGrant,
}

/// 人工或策略授权远程控制请求的结果。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PermissionGrant {
    pub accepted: bool,
    pub reason: Option<String>,
}

/// 文件传输块。
///
/// 元数据放在 `envelope`，真实字节放在 `bytes`，方便后续替换为二进制通道。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileChunk {
    pub envelope: FileTransferEnvelope,
    pub bytes: Vec<u8>,
}

/// 视频帧传输块。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VideoChunk {
    pub envelope: VideoFrameEnvelope,
    pub bytes: Vec<u8>,
}

/// 远程桌面控制协议的 JSON 控制帧。
///
/// 该 enum 是当前 wire contract 的中心；新增变体需要考虑旧客户端的反序列化兼容性。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ControlFrame {
    Hello(DeviceHello),
    DeviceSnapshot(Vec<DeviceDescriptor>),
    SessionOffer(SessionOffer),
    SessionAccept(SessionAccept),
    PermissionGrant(PermissionGrant),
    ClipboardSync(ClipboardPayload),
    FileChunk(FileChunk),
    InputEvent(RemoteInputEvent),
    VideoChunk(VideoChunk),
    Heartbeat,
    Error { code: String, message: String },
}

impl ControlFrame {
    /// 将控制帧编码为 JSON 字节。
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("serialize remote control frame")
    }

    /// 从 JSON 字节恢复控制帧。
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("deserialize remote control frame")
    }
}
