//! 基于 QUIC 的远程会话中继服务。
//!
//! 本模块承载 crate 的公共 API；crate root 只负责 `automod` 模块收集。
//!
//! 提供设备注册、会话请求/授权、剪贴板同步、视频帧推送和文件传输暂存等能力，
//! 用于在远端设备之间建立受控的远程桌面会话。
//!
//! ## 核心类型
//!
//! - [`RemoteRelayService`]：中继服务主入口，管理设备与会话的生命周期。
//! - [`RelayRuntimeConfig`]：运行时配置（绑定地址、最大并发数、空闲超时）。
//! - 远程会话操作直接返回 [`anyhow::Result`]，查找和授权失败用错误消息携带上下文。

use anyhow::{anyhow, bail};
use az_derive_aliases::{apply, impl_default, plain_clone_debug, plain_default_debug};
use az_remote_model::api::{
    ClipboardPayload, DeviceDescriptor, DeviceId, FileTransferEnvelope, OnlineStatus,
    SessionCapability, SessionGrant, SessionId, SessionRequest, SessionState, SessionSummary,
    VideoFrameEnvelope,
};
use chrono::Utc;
use quinn::VarInt;
use std::collections::HashMap;
use uuid::Uuid;

/// QUIC 中继运行时配置。
#[apply(plain_clone_debug)]
pub struct RelayRuntimeConfig {
    /// 中继服务监听地址。
    pub bind_addr: String,
    /// 允许的最大并发会话数。
    pub max_concurrent_sessions: u32,
    /// 空闲会话超时，单位秒。
    pub idle_timeout_secs: u64,
}

impl_default!(RelayRuntimeConfig => RelayRuntimeConfig {
    bind_addr: "127.0.0.1:7443".into(),
    max_concurrent_sessions: VarInt::from_u32(64).into_inner() as u32,
    idle_timeout_secs: 30,
});

/// 内存态远程会话中继服务。
///
/// 该类型只维护当前进程内的设备和会话快照；正式持久化、鉴权和网络 IO 由外层服务接入。
#[apply(plain_default_debug)]
pub struct RemoteRelayService {
    devices: HashMap<DeviceId, DeviceDescriptor>,
    sessions: HashMap<SessionId, SessionSummary>,
}

impl RemoteRelayService {
    /// 创建空的中继服务状态。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册或覆盖设备，并把设备标记为在线。
    pub fn register_device(&mut self, mut device: DeviceDescriptor) -> DeviceDescriptor {
        device.online_status = OnlineStatus::Online;
        device.last_seen_at = Utc::now();
        self.devices.insert(device.device_id, device.clone());
        device
    }

    /// 列出当前注册设备，按设备名称排序。
    pub fn list_devices(&self) -> Vec<DeviceDescriptor> {
        let mut devices = self.devices.values().cloned().collect::<Vec<_>>();
        devices.sort_by(|left, right| left.device_name.cmp(&right.device_name));
        devices
    }

    /// 创建 viewer 到 host 的会话请求。
    ///
    /// viewer 和 host 都必须已经注册；新会话初始状态为 `Requested`。
    pub fn request_session(
        &mut self,
        viewer_id: DeviceId,
        host_id: DeviceId,
        capability: SessionCapability,
    ) -> anyhow::Result<SessionRequest> {
        self.devices
            .get(&viewer_id)
            .ok_or_else(|| anyhow!("device `{viewer_id}` was not found"))?;
        self.devices
            .get(&host_id)
            .ok_or_else(|| anyhow!("device `{host_id}` was not found"))?;

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

    /// host 对会话请求做授权决策。
    ///
    /// 接受时会话进入 `Active`；拒绝时会话进入 `Rejected` 并返回错误。
    pub fn grant_session(
        &mut self,
        session_id: SessionId,
        accepted: bool,
        reason: Option<String>,
    ) -> anyhow::Result<SessionGrant> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session `{session_id}` was not found"))?;
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
            let reason = grant.reason.clone().unwrap_or_else(|| "rejected".into());
            bail!("host `{}` rejected the request: {reason}", grant.session_id);
        }
        Ok(grant)
    }

    /// 更新会话中的最新剪贴板载荷。
    pub fn push_clipboard(
        &mut self,
        session_id: SessionId,
        clipboard: ClipboardPayload,
    ) -> anyhow::Result<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session `{session_id}` was not found"))?;
        summary.clipboard = Some(clipboard);
        Ok(())
    }

    /// 更新会话中的最新视频帧元数据。
    pub fn push_frame(
        &mut self,
        session_id: SessionId,
        frame: VideoFrameEnvelope,
    ) -> anyhow::Result<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session `{session_id}` was not found"))?;
        summary.latest_frame = Some(frame);
        Ok(())
    }

    /// 暂存会话中的待处理文件传输元数据。
    pub fn stage_file_transfer(
        &mut self,
        session_id: SessionId,
        transfer: FileTransferEnvelope,
    ) -> anyhow::Result<()> {
        let summary = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session `{session_id}` was not found"))?;
        summary.pending_transfer = Some(transfer);
        Ok(())
    }

    /// 获取指定会话的当前快照。
    pub fn session_summary(&self, session_id: SessionId) -> anyhow::Result<SessionSummary> {
        self.sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| anyhow!("session `{session_id}` was not found"))
    }
}
