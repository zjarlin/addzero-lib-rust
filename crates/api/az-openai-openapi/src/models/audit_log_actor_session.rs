// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogActorSession` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogActorUser,
};

/// The session in which the audit logged action was performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogActorSession {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<AuditLogActorUser>,
    /// The IP address from which the action was performed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}
