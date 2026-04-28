#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub type DeviceId = Uuid;
pub type SessionId = Uuid;
pub type TransferId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemotePlatform {
    MacOs,
    Windows,
    LinuxX11,
    LinuxWayland,
    Browser,
}

impl fmt::Display for RemotePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::MacOs => "macOS",
            Self::Windows => "Windows",
            Self::LinuxX11 => "Linux (X11)",
            Self::LinuxWayland => "Linux (Wayland)",
            Self::Browser => "Browser",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceRole {
    Viewer,
    Host,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnlineStatus {
    Online,
    Idle,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRequest {
    pub session_id: SessionId,
    pub viewer_id: DeviceId,
    pub host_id: DeviceId,
    pub capability: SessionCapability,
    pub requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionGrant {
    pub session_id: SessionId,
    pub host_id: DeviceId,
    pub accepted: bool,
    pub reason: Option<String>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    Down,
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardPayload {
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTransferEnvelope {
    pub transfer_id: TransferId,
    pub session_id: SessionId,
    pub file_name: String,
    pub total_bytes: u64,
    pub chunk_index: u32,
    pub chunk_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    JpegFrames,
    PngFrames,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoFrameEnvelope {
    pub session_id: SessionId,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub sequence: u64,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Requested,
    Active,
    Rejected,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub viewer_id: DeviceId,
    pub host_id: DeviceId,
    pub state: SessionState,
    pub clipboard: Option<ClipboardPayload>,
    pub latest_frame: Option<VideoFrameEnvelope>,
    pub pending_transfer: Option<FileTransferEnvelope>,
}
