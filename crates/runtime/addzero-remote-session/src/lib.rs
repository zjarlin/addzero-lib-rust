#![forbid(unsafe_code)]

use addzero_remote_model::{
    ClipboardPayload, DeviceDescriptor, DeviceId, FileTransferEnvelope, OnlineStatus, SessionGrant,
    SessionId, SessionRequest, SessionState, SessionSummary, VideoFrameEnvelope,
};
use chrono::Utc;
use quinn::VarInt;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub type RemoteSessionResult<T> = Result<T, RemoteSessionError>;

#[derive(Debug, Error)]
pub enum RemoteSessionError {
    #[error("device `{0}` was not found")]
    DeviceNotFound(DeviceId),
    #[error("session `{0}` was not found")]
    SessionNotFound(SessionId),
    #[error("host `{0}` rejected the request: {1}")]
    SessionRejected(SessionId, String),
}

#[derive(Debug, Clone)]
pub struct RelayRuntimeConfig {
    pub bind_addr: String,
    pub max_concurrent_sessions: u32,
    pub idle_timeout_secs: u64,
}

impl Default for RelayRuntimeConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:7443".into(),
            max_concurrent_sessions: VarInt::from_u32(64).into_inner() as u32,
            idle_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Default)]
pub struct RemoteRelayService {
    devices: HashMap<DeviceId, DeviceDescriptor>,
    sessions: HashMap<SessionId, SessionSummary>,
}

impl RemoteRelayService {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_device(&mut self, mut device: DeviceDescriptor) -> DeviceDescriptor {
        device.online_status = OnlineStatus::Online;
        device.last_seen_at = Utc::now();
        self.devices.insert(device.device_id, device.clone());
        device
    }

    pub fn list_devices(&self) -> Vec<DeviceDescriptor> {
        let mut devices = self.devices.values().cloned().collect::<Vec<_>>();
        devices.sort_by(|left, right| left.device_name.cmp(&right.device_name));
        devices
    }

    pub fn request_session(
        &mut self,
        viewer_id: DeviceId,
        host_id: DeviceId,
        capability: addzero_remote_model::SessionCapability,
    ) -> RemoteSessionResult<SessionRequest> {
        self.devices
            .get(&viewer_id)
            .ok_or(RemoteSessionError::DeviceNotFound(viewer_id))?;
        self.devices
            .get(&host_id)
            .ok_or(RemoteSessionError::DeviceNotFound(host_id))?;

        let request = SessionRequest {
            session_id: Uuid::new_v4(),
            viewer_id,
            host_id,
            capability,
            requested_at: Utc::now(),
        };
        self.sessions.insert(
            request.session_id,
            SessionSummary {
                session_id: request.session_id,
                viewer_id,
                host_id,
                state: SessionState::Requested,
                clipboard: None,
                latest_frame: None,
                pending_transfer: None,
            },
        );
        Ok(request)
    }

    pub fn grant_session(
        &mut self,
        session_id: SessionId,
        accepted: bool,
        reason: Option<String>,
    ) -> RemoteSessionResult<SessionGrant> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or(RemoteSessionError::SessionNotFound(session_id))?;
        summary.state = if accepted {
            SessionState::Active
        } else {
            SessionState::Rejected
        };
        let grant = SessionGrant {
            session_id,
            host_id: summary.host_id,
            accepted,
            reason,
            granted_at: Utc::now(),
        };
        if !grant.accepted {
            return Err(RemoteSessionError::SessionRejected(
                grant.session_id,
                grant.reason.clone().unwrap_or_else(|| "rejected".into()),
            ));
        }
        Ok(grant)
    }

    pub fn push_clipboard(
        &mut self,
        session_id: SessionId,
        clipboard: ClipboardPayload,
    ) -> RemoteSessionResult<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or(RemoteSessionError::SessionNotFound(session_id))?;
        summary.clipboard = Some(clipboard);
        Ok(())
    }

    pub fn push_frame(
        &mut self,
        session_id: SessionId,
        frame: VideoFrameEnvelope,
    ) -> RemoteSessionResult<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or(RemoteSessionError::SessionNotFound(session_id))?;
        summary.latest_frame = Some(frame);
        Ok(())
    }

    pub fn stage_file_transfer(
        &mut self,
        session_id: SessionId,
        transfer: FileTransferEnvelope,
    ) -> RemoteSessionResult<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or(RemoteSessionError::SessionNotFound(session_id))?;
        summary.pending_transfer = Some(transfer);
        Ok(())
    }

    pub fn session_summary(&self, session_id: SessionId) -> RemoteSessionResult<SessionSummary> {
        self.sessions
            .get(&session_id)
            .cloned()
            .ok_or(RemoteSessionError::SessionNotFound(session_id))
    }
}
