use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentArtifactChannel {
    MacosBinary,
    DockerCompose,
}

impl AgentArtifactChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MacosBinary => "macos_binary",
            Self::DockerCompose => "docker_compose",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairingStatus {
    Pending,
    Approved,
    Exchanged,
    Expired,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentNodeStatus {
    Pending,
    Online,
    Offline,
    Revoked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    UseWeb,
    UseAgent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PairingRequest {
    pub channel: AgentArtifactChannel,
    pub device_name: String,
    pub platform: String,
    pub agent_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PairingCreateResponse {
    pub session: PairingSessionSummary,
    pub poll_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PairingExchangeRequest {
    pub poll_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PairingExchangeResponse {
    pub node: AgentNode,
    pub node_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentHeartbeat {
    pub node_token: String,
    pub platform: String,
    pub agent_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillSnapshot {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
    pub content_hash: String,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillSyncRequest {
    pub node_token: String,
    pub fs_root: String,
    pub skills: Vec<SkillSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillSyncResponse {
    pub node: AgentNode,
    pub uploaded_names: Vec<String>,
    pub download_skills: Vec<SkillSnapshot>,
    pub conflicts: Vec<SkillConflict>,
    pub synced_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: ConflictResolution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentRuntimeOverview {
    pub artifacts: Vec<AgentArtifact>,
    pub active_node: Option<AgentNode>,
    pub pairing_sessions: Vec<PairingSessionSummary>,
    pub conflicts: Vec<SkillConflict>,
    pub fs_root: String,
    pub pg_online: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub authenticated: bool,
    pub username: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
