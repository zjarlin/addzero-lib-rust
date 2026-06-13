// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `PublicCreateOrganizationRoleBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for creating a custom role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicCreateOrganizationRoleBody {
    /// Unique name for the role.
    pub role_name: String,
    /// Permissions to grant to the role.
    pub permissions: Vec<String>,
    /// Optional description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
