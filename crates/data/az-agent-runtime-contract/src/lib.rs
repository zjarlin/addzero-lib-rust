//! Agent 运行时与服务端之间的 API 契约类型定义。
//!
//! 定义了 Agent 配对（pairing）、节点管理、技能同步、冲突解决及会话认证等
//! 交互流程中使用的所有请求/响应数据结构和枚举类型。
//!
//! # 核心枚举
//!
//! - [`AgentArtifactChannel`] — Agent 分发渠道（macOS 二进制、Docker Compose）。
//! - [`PairingStatus`] — 配对会话状态（待审批、已批准、已交换密钥、已过期、已撤销）。
//! - [`AgentNodeStatus`] — Agent 节点在线状态（待确认、在线、离线、已撤销）。
//! - [`ConflictResolution`] — 技能冲突解决策略（以 Web 端为准或以 Agent 端为准）。
//!
//! # 关键结构体
//!
//! - **配对流程**：[`PairingRequest`]、[`PairingSessionSummary`]、[`PairingCreateResponse`]、
//!   [`PairingExchangeRequest`]、[`PairingExchangeResponse`]。
//! - **节点管理**：[`AgentNode`]、[`AgentHeartbeat`]。
//! - **技能同步**：[`SkillSnapshot`]、[`SkillSyncRequest`]、[`SkillSyncResponse`]、
//!   [`SkillConflict`]、[`ResolveConflictRequest`]。
//! - **汇总视图**：[`AgentRuntimeOverview`] — 聚合制品、节点、配对会话和冲突的总览。
//! - **认证**：[`SessionUser`]、[`LoginRequest`]。
//! - **制品**：[`AgentArtifact`] — Agent 安装包元数据（下载地址、校验和、安装/启动/卸载命令）。

use az_derive_aliases::{apply, serde_code_enum, serde_eq};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[apply(serde_code_enum)]
pub enum AgentArtifactChannel {
    MacosBinary,
    DockerCompose,
}

impl AgentArtifactChannel {
    pub fn as_str(self) -> &'static str {
        self.code()
    }
}

#[apply(serde_code_enum)]
pub enum PairingStatus {
    Pending,
    Approved,
    Exchanged,
    Expired,
    Revoked,
}

#[apply(serde_code_enum)]
pub enum AgentNodeStatus {
    Pending,
    Online,
    Offline,
    Revoked,
}

#[apply(serde_code_enum)]
pub enum ConflictResolution {
    UseWeb,
    UseAgent,
}

#[apply(serde_eq)]
pub struct AgentArtifact {
    pub id: Uuid,
    pub channel: AgentArtifactChannel,
    pub title: String,
    pub version: String,
    pub platform: String,
    pub package_format: String,
    pub download_url: String,
    pub checksum: String,
    pub install_command: String,
    pub launch_command: String,
    pub uninstall_command: String,
    pub service_name: String,
    pub note: String,
    pub active: bool,
}

#[apply(serde_eq)]
pub struct PairingRequest {
    pub channel: AgentArtifactChannel,
    pub device_name: String,
    pub platform: String,
    pub agent_version: String,
}

#[apply(serde_eq)]
pub struct PairingSessionSummary {
    pub id: Uuid,
    pub channel: AgentArtifactChannel,
    pub device_name: String,
    pub platform: String,
    pub agent_version: String,
    pub status: PairingStatus,
    pub approve_url: String,
    pub expires_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub exchanged_at: Option<DateTime<Utc>>,
}

#[apply(serde_eq)]
pub struct PairingCreateResponse {
    pub session: PairingSessionSummary,
    pub poll_token: String,
}

#[apply(serde_eq)]
pub struct PairingExchangeRequest {
    pub poll_token: String,
}

#[apply(serde_eq)]
pub struct AgentNode {
    pub id: Uuid,
    pub display_name: String,
    pub platform: String,
    pub channel: AgentArtifactChannel,
    pub agent_version: String,
    pub status: AgentNodeStatus,
    pub paired_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_uploaded_count: usize,
    pub last_downloaded_count: usize,
    pub last_conflict_count: usize,
}

#[apply(serde_eq)]
pub struct PairingExchangeResponse {
    pub node: AgentNode,
    pub node_token: String,
}

#[apply(serde_eq)]
pub struct AgentHeartbeat {
    pub node_token: String,
    pub platform: String,
    pub agent_version: String,
}

#[apply(serde_eq)]
pub struct SkillSnapshot {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
    pub content_hash: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[apply(serde_eq)]
pub struct SkillSyncRequest {
    pub node_token: String,
    pub fs_root: String,
    pub skills: Vec<SkillSnapshot>,
}

#[apply(serde_eq)]
pub struct SkillConflict {
    pub id: Uuid,
    pub node_id: Uuid,
    pub skill_name: String,
    pub server_hash: String,
    pub agent_hash: String,
    pub server_updated_at: Option<DateTime<Utc>>,
    pub agent_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<ConflictResolution>,
}

#[apply(serde_eq)]
pub struct SkillSyncResponse {
    pub node: AgentNode,
    pub uploaded_names: Vec<String>,
    pub download_skills: Vec<SkillSnapshot>,
    pub conflicts: Vec<SkillConflict>,
    pub synced_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct ResolveConflictRequest {
    pub resolution: ConflictResolution,
}

#[apply(serde_eq)]
pub struct AgentRuntimeOverview {
    pub artifacts: Vec<AgentArtifact>,
    pub active_node: Option<AgentNode>,
    pub pairing_sessions: Vec<PairingSessionSummary>,
    pub conflicts: Vec<SkillConflict>,
    pub fs_root: String,
    pub pg_online: bool,
}

#[apply(serde_eq)]
pub struct SessionUser {
    pub authenticated: bool,
    pub username: Option<String>,
}

#[apply(serde_eq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::{AgentArtifactChannel, AgentNodeStatus, PairingStatus};

    #[test]
    fn contract_enums_keep_snake_case_wire_shape() {
        assert_eq!(AgentArtifactChannel::MacosBinary.as_str(), "macos_binary");
        assert_eq!(
            AgentArtifactChannel::from_code("docker_compose"),
            Some(AgentArtifactChannel::DockerCompose)
        );
        assert_eq!(
            serde_json::to_string(&PairingStatus::Approved)
                .expect("pairing status should serialize"),
            "\"approved\""
        );
        assert_eq!(
            serde_json::to_string(&AgentNodeStatus::Offline).expect("node status should serialize"),
            "\"offline\""
        );
    }
}
