// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatSessionResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatSessionChatkitConfiguration,
    ChatSessionRateLimits,
    ChatSessionStatus,
    ChatkitWorkflow,
};

/// Represents a ChatKit session and its resolved configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionResource {
    /// Identifier for the ChatKit session.
    pub id: String,
    /// Type discriminator that is always `chatkit.session`.
    pub object: String,
    /// Unix timestamp (in seconds) for when the session expires.
    pub expires_at: i64,
    /// Ephemeral client secret that authenticates session requests.
    pub client_secret: String,
    /// Workflow metadata for the session.
    pub workflow: ChatkitWorkflow,
    /// User identifier associated with the session.
    pub user: String,
    /// Resolved rate limit values.
    pub rate_limits: ChatSessionRateLimits,
    /// Convenience copy of the per-minute request limit.
    pub max_requests_per_1_minute: i32,
    /// Current lifecycle state of the session.
    pub status: ChatSessionStatus,
    /// Resolved ChatKit feature configuration for the session.
    pub chatkit_configuration: ChatSessionChatkitConfiguration,
}
