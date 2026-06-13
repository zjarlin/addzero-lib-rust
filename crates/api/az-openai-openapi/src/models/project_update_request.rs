// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectUpdateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdateRequest {
    /// The updated name of the project, this name appears in reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// External key ID to associate with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: Option<String>,
    /// Geography for the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geography: Option<String>,
}
