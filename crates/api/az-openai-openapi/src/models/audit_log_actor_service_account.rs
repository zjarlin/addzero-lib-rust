// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogActorServiceAccount` DTO.

use serde::{Deserialize, Serialize};

/// The service account that performed the audit logged action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogActorServiceAccount {
    /// The service account id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
