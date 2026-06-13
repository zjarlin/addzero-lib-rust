// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogApiKeyUpdated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogApiKeyUpdatedChangesRequested,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogApiKeyUpdated {
    /// The tracking ID of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to update the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes_requested: Option<AuditLogApiKeyUpdatedChangesRequested>,
}
