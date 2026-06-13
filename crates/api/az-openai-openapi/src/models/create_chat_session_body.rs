// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatSessionBody` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatkitConfigurationParam,
    ExpiresAfterParam,
    RateLimitsParam,
    WorkflowParam,
};

/// Parameters for provisioning a new ChatKit session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatSessionBody {
    /// Workflow that powers the session.
    pub workflow: WorkflowParam,
    /// A free-form string that identifies your end user; ensures this Session can access other objects that
    /// have the same `user` scope.
    pub user: String,
    /// Optional override for session expiration timing in seconds from creation. Defaults to 10 minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<ExpiresAfterParam>,
    /// Optional override for per-minute request limits. When omitted, defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimitsParam>,
    /// Optional overrides for ChatKit runtime configuration features
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatkit_configuration: Option<ChatkitConfigurationParam>,
}
