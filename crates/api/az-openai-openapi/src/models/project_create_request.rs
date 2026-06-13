// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectCreateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreateRequest {
    /// The friendly name of the project, this name appears in reports.
    pub name: String,
    /// Create the project with the specified data residency region. Your organization must have access to
    /// Data residency functionality in order to use. See [data residency controls](/docs/guides/your-
    /// data#data-residency-controls) to review the functionality and limitations of setting this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geography: Option<String>,
    /// External key ID to associate with the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_key_id: Option<String>,
}
