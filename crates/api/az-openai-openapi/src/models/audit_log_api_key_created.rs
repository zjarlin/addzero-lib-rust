// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogApiKeyCreated` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AuditLogApiKeyCreatedData,
};

/// The details for events with this `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogApiKeyCreated {
    /// The tracking ID of the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The payload used to create the API key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<AuditLogApiKeyCreatedData>,
}
