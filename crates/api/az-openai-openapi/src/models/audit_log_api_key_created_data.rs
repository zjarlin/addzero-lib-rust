// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogApiKeyCreatedData` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to create the API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogApiKeyCreatedData {
    /// A list of scopes allowed for the API key, e.g. `["api.model.request"]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}
