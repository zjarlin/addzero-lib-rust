#![forbid(unsafe_code)]

//! 远程桌面控制的数据模型定义库。
//!
//! 定义远程桌面会话所需的全部协议数据类型，包括设备描述、会话能力协商、
//! 输入事件（鼠标/键盘）、剪贴板同步、文件传输信封、视频帧信封等。
//! 这些类型可序列化为 JSON，用于客户端与服务端之间的消息交换。
//!
//! ## 主要类型
//!
//! - [`DeviceDescriptor`] — 设备信息（ID、名称、平台、角色、能力、在线状态）。
//! - [`SessionRequest`] / [`SessionGrant`] — 会话建立的请求与授权流程。
//! - [`SessionSummary`] — 会话完整状态快照，包含最新视频帧与剪贴板内容。
//! - [`RemoteInputEvent`] — 远程输入事件枚举：鼠标移动、按键、滚轮、文本输入。
//! - [`VideoFrameEnvelope`] / [`FileTransferEnvelope`] — 视频帧与文件分块传输的信封结构。
//! - [`ClipboardPayload`] — 剪贴板同步载荷。
//!
//! ## 平台与角色
//!
//! [`RemotePlatform`] 支持 macOS、Windows、Linux (X11/Wayland)、Browser；
//! [`DeviceRole`] 区分 Host（被控端）与 Viewer（控制端）。

use az_derive_aliases::{apply, serde_eq, serde_eq_copy, serde_eq_copy_display};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub type DeviceId = Uuid;
pub type SessionId = Uuid;
pub type TransferId = Uuid;

#[apply(serde_eq_copy_display)]
pub enum RemotePlatform {
    #[display("macOS")]
    MacOs,
    #[display("Windows")]
    Windows,
    #[display("Linux (X11)")]
    LinuxX11,
    #[display("Linux (Wayland)")]
    LinuxWayland,
    #[display("Browser")]
    Browser,
}

#[apply(serde_eq_copy)]
pub enum DeviceRole {
    Viewer,
    Host,
}

#[apply(serde_eq_copy)]
pub enum OnlineStatus {
    Online,
    Idle,
    Busy,
    Offline,
}

#[apply(serde_eq_copy)]
pub struct SessionCapability {
    pub screen: bool,
    pub input_control: bool,
    pub clipboard_text: bool,
    pub file_transfer: bool,
}

impl SessionCapability {
    #[must_use]
    pub fn full_host() -> Self {
        Self {
            screen: true,
            input_control: true,
            clipboard_text: true,
            file_transfer: true,
        }
    }

    #[must_use]
    pub fn web_viewer() -> Self {
        Self {
            screen: true,
            input_control: true,
            clipboard_text: true,
            file_transfer: false,
        }
    }
}

#[apply(serde_eq)]
pub struct DeviceDescriptor {
    pub device_id: DeviceId,
    pub device_name: String,
    pub platform: RemotePlatform,
    pub role: DeviceRole,
    pub capabilities: SessionCapability,
    pub online_status: OnlineStatus,
    pub last_seen_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[apply(serde_eq)]
pub struct SessionRequest {
    pub session_id: SessionId,
    pub viewer_id: DeviceId,
    pub host_id: DeviceId,
    pub capability: SessionCapability,
    pub requested_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct SessionGrant {
    pub session_id: SessionId,
    pub host_id: DeviceId,
    pub accepted: bool,
    pub reason: Option<String>,
    pub granted_at: DateTime<Utc>,
}

#[apply(serde_eq_copy)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
}

#[apply(serde_eq_copy)]
pub enum KeyState {
    Down,
    Up,
}

#[apply(serde_eq)]
pub enum RemoteInputEvent {
    PointerMove {
        x: u16,
        y: u16,
    },
    PointerButton {
        button: PointerButton,
        state: KeyState,
    },
    PointerScroll {
        delta_x: i16,
        delta_y: i16,
    },
    Key {
        key: String,
        state: KeyState,
    },
    Text {
        text: String,
    },
}

#[apply(serde_eq)]
pub struct ClipboardPayload {
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct FileTransferEnvelope {
    pub transfer_id: TransferId,
    pub session_id: SessionId,
    pub file_name: String,
    pub total_bytes: u64,
    pub chunk_index: u32,
    pub chunk_count: u32,
}

#[apply(serde_eq_copy)]
pub enum VideoCodec {
    JpegFrames,
    PngFrames,
}

#[apply(serde_eq)]
pub struct VideoFrameEnvelope {
    pub session_id: SessionId,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub sequence: u64,
    pub captured_at: DateTime<Utc>,
}

#[apply(serde_eq_copy)]
pub enum SessionState {
    Requested,
    Active,
    Rejected,
    Closed,
}

#[apply(serde_eq)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub viewer_id: DeviceId,
    pub host_id: DeviceId,
    pub state: SessionState,
    pub clipboard: Option<ClipboardPayload>,
    pub latest_frame: Option<VideoFrameEnvelope>,
    pub pending_transfer: Option<FileTransferEnvelope>,
}
