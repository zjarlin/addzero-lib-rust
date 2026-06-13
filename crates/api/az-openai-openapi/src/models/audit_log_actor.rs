// Generated from OpenAPI spec. Do not edit by hand.
//! `AuditLogActor` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogActorApiKey,
    AuditLogActorSession,
};

/// The actor who performed the audit logged action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogActor {
    /// The type of actor. Is either `session` or `api_key`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<AuditLogActorSession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<AuditLogActorApiKey>,
}
