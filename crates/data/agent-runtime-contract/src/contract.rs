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

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Agent 安装制品分发渠道。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentArtifactChannel {
    MacosBinary,
    DockerCompose,
}

impl AgentArtifactChannel {
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

/// Agent 配对会话状态。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PairingStatus {
    Pending,
    Approved,
    Exchanged,
    Expired,
    Revoked,
}

impl PairingStatus {
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

/// Agent 节点状态。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AgentNodeStatus {
    Pending,
    Online,
    Offline,
    Revoked,
}

impl AgentNodeStatus {
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

/// 技能同步冲突解决策略。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConflictResolution {
    UseWeb,
    UseAgent,
}

impl ConflictResolution {
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

/// 可供用户下载安装的 Agent 制品元数据。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentArtifact {
    /// 制品 id。
    pub id: Uuid,
    /// 分发渠道。
    pub channel: AgentArtifactChannel,
    /// 展示标题。
    pub title: String,
    /// Agent 版本。
    pub version: String,
    /// 目标平台。
    pub platform: String,
    /// 包格式。
    pub package_format: String,
    /// 下载地址。
    pub download_url: String,
    /// 校验和。
    pub checksum: String,
    /// 安装命令。
    pub install_command: String,
    /// 启动命令。
    pub launch_command: String,
    /// 卸载命令。
    pub uninstall_command: String,
    /// 系统服务名。
    pub service_name: String,
    /// 补充说明。
    pub note: String,
    /// 是否为当前可用制品。
    pub active: bool,
}

/// Agent 发起配对时提交的请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PairingRequest {
    /// Agent 来源渠道。
    pub channel: AgentArtifactChannel,
    /// 本机展示名。
    pub device_name: String,
    /// 本机平台标识。
    pub platform: String,
    /// Agent 版本。
    pub agent_version: String,
}

/// 配对会话摘要。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PairingSessionSummary {
    /// 配对会话 id。
    pub id: Uuid,
    /// Agent 来源渠道。
    pub channel: AgentArtifactChannel,
    /// 本机展示名。
    pub device_name: String,
    /// 本机平台标识。
    pub platform: String,
    /// Agent 版本。
    pub agent_version: String,
    /// 当前配对状态。
    pub status: PairingStatus,
    /// Web 端审批地址。
    pub approve_url: String,
    /// 过期时间。
    pub expires_at: DateTime<Utc>,
    /// 审批通过时间。
    pub approved_at: Option<DateTime<Utc>>,
    /// token 交换完成时间。
    pub exchanged_at: Option<DateTime<Utc>>,
}

/// 创建配对会话后的响应。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PairingCreateResponse {
    /// 配对会话摘要。
    pub session: PairingSessionSummary,
    /// Agent 轮询审批结果使用的临时 token。
    pub poll_token: String,
}

/// Agent 交换节点 token 的请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PairingExchangeRequest {
    /// 创建配对时拿到的轮询 token。
    pub poll_token: String,
}

/// 已配对的 Agent 节点。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentNode {
    /// 节点 id。
    pub id: Uuid,
    /// 展示名称。
    pub display_name: String,
    /// 节点平台。
    pub platform: String,
    /// Agent 来源渠道。
    pub channel: AgentArtifactChannel,
    /// Agent 版本。
    pub agent_version: String,
    /// 节点状态。
    pub status: AgentNodeStatus,
    /// 完成配对时间。
    pub paired_at: DateTime<Utc>,
    /// 最近心跳时间。
    pub last_seen_at: Option<DateTime<Utc>>,
    /// 最近同步时间。
    pub last_sync_at: Option<DateTime<Utc>>,
    /// 最近一次上传技能数量。
    pub last_uploaded_count: usize,
    /// 最近一次下载技能数量。
    pub last_downloaded_count: usize,
    /// 最近一次冲突数量。
    pub last_conflict_count: usize,
}

/// 配对 token 交换完成后的响应。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PairingExchangeResponse {
    /// 配对完成后的节点摘要。
    pub node: AgentNode,
    /// Agent 后续心跳和同步使用的节点 token。
    pub node_token: String,
}

/// Agent 心跳请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentHeartbeat {
    /// 节点 token。
    pub node_token: String,
    /// 节点当前平台标识。
    pub platform: String,
    /// Agent 当前版本。
    pub agent_version: String,
}

/// 单个技能文件的同步快照。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillSnapshot {
    /// 技能名称。
    pub name: String,
    /// 技能关键词。
    pub keywords: Vec<String>,
    /// 技能描述。
    pub description: String,
    /// 技能文件正文。
    pub body: String,
    /// 内容哈希。
    pub content_hash: String,
    /// 文件或服务端记录更新时间。
    pub updated_at: Option<DateTime<Utc>>,
}

/// Agent 上传技能快照并拉取服务端变更的同步请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillSyncRequest {
    /// 节点 token。
    pub node_token: String,
    /// Agent 本地技能根目录。
    pub fs_root: String,
    /// Agent 当前看到的技能快照。
    pub skills: Vec<SkillSnapshot>,
}

/// 服务端检测到的技能同步冲突。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillConflict {
    /// 冲突 id。
    pub id: Uuid,
    /// 关联节点 id。
    pub node_id: Uuid,
    /// 冲突技能名。
    pub skill_name: String,
    /// 服务端内容哈希。
    pub server_hash: String,
    /// Agent 端内容哈希。
    pub agent_hash: String,
    /// 服务端更新时间。
    pub server_updated_at: Option<DateTime<Utc>>,
    /// Agent 端更新时间。
    pub agent_updated_at: Option<DateTime<Utc>>,
    /// 冲突创建时间。
    pub created_at: DateTime<Utc>,
    /// 冲突解决时间。
    pub resolved_at: Option<DateTime<Utc>>,
    /// 已选择的解决策略。
    pub resolution: Option<ConflictResolution>,
}

/// 技能同步响应。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SkillSyncResponse {
    /// 更新后的节点摘要。
    pub node: AgentNode,
    /// 本次服务端接受的上传技能名。
    pub uploaded_names: Vec<String>,
    /// Agent 需要下载或覆盖的技能快照。
    pub download_skills: Vec<SkillSnapshot>,
    /// 本次检测到或仍未解决的冲突。
    pub conflicts: Vec<SkillConflict>,
    /// 同步完成时间。
    pub synced_at: DateTime<Utc>,
}

/// 解决单个技能冲突的请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolveConflictRequest {
    /// 冲突解决策略。
    pub resolution: ConflictResolution,
}

/// Agent 运行时总览视图。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentRuntimeOverview {
    /// 可用 Agent 安装制品。
    pub artifacts: Vec<AgentArtifact>,
    /// 当前会话关联的活跃节点。
    pub active_node: Option<AgentNode>,
    /// 最近配对会话。
    pub pairing_sessions: Vec<PairingSessionSummary>,
    /// 未解决或最近冲突。
    pub conflicts: Vec<SkillConflict>,
    /// 服务端配置的技能根路径。
    pub fs_root: String,
    /// PostgreSQL 是否在线。
    pub pg_online: bool,
}

/// 当前 Web 会话用户摘要。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionUser {
    /// 当前会话是否已认证。
    pub authenticated: bool,
    /// 当前用户名。
    pub username: Option<String>,
}

/// 登录请求。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
    /// 登录用户名。
    pub username: String,
    /// 登录密码。
    pub password: String,
}
