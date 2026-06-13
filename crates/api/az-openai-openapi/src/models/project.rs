// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Project` DTO.

use serde::{Deserialize, Serialize};

/// Represents an individual project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The object type, which is always `organization.project`
    pub object: String,
    /// The name of the project. This appears in reporting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The Unix timestamp (in seconds) of when the project was created.
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    /// `active` or `archived`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The external key associated with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: Option<String>,
}
