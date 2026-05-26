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
//! - [`ProtocolError`] — 协议错误类型，目前仅封装 JSON 序列化/反序列化失败。
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
//! use az_remote_protocol::{ControlFrame, DeviceHello, StreamKind};
//! use az_remote_model::{DeviceDescriptor, DeviceRole, OnlineStatus, RemotePlatform, SessionCapability};
//! use chrono::Utc;
//! use uuid::Uuid;
//!
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
//! let bytes = frame.to_json_bytes().unwrap();
//! let restored = ControlFrame::from_json_bytes(&bytes).unwrap();
//! assert_eq!(frame, restored);
//! ```
#![forbid(unsafe_code)]

use az_derive_aliases::{apply, error, serde_code_enum, serde_eq, serde_eq_redacted};
use az_remote_model::{
    ClipboardPayload, DeviceDescriptor, FileTransferEnvelope, RemoteInputEvent, SessionGrant,
    SessionRequest, VideoFrameEnvelope,
};

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[apply(error)]
pub enum ProtocolError {
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[apply(serde_code_enum)]
pub enum StreamKind {
    Control,
    Video,
    Input,
    Clipboard,
    File,
}

#[apply(serde_eq_redacted)]
pub struct DeviceHello {
    pub device: DeviceDescriptor,
    #[debug(skip)]
    pub relay_token: String,
}

#[apply(serde_eq)]
pub struct SessionOffer {
    pub request: SessionRequest,
}

#[apply(serde_eq)]
pub struct SessionAccept {
    pub grant: SessionGrant,
}

#[apply(serde_eq)]
pub struct PermissionGrant {
    pub accepted: bool,
    pub reason: Option<String>,
}

#[apply(serde_eq)]
pub struct FileChunk {
    pub envelope: FileTransferEnvelope,
    pub bytes: Vec<u8>,
}

#[apply(serde_eq)]
pub struct VideoChunk {
    pub envelope: VideoFrameEnvelope,
    pub bytes: Vec<u8>,
}

#[apply(serde_eq)]
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
    pub fn to_json_bytes(&self) -> ProtocolResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(ProtocolError::from)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> ProtocolResult<Self> {
        serde_json::from_slice(bytes).map_err(ProtocolError::from)
    }
}
