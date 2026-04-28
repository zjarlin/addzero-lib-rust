#![forbid(unsafe_code)]

use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, FileTransferEnvelope, RemoteInputEvent, SessionGrant,
    SessionRequest, VideoFrameEnvelope,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ProtocolResult<T> = Result<T, ProtocolError>;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamKind {
    Control,
    Video,
    Input,
    Clipboard,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceHello {
    pub device: DeviceDescriptor,
    pub relay_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionOffer {
    pub request: SessionRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionAccept {
    pub grant: SessionGrant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub accepted: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChunk {
    pub envelope: FileTransferEnvelope,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoChunk {
    pub envelope: VideoFrameEnvelope,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
