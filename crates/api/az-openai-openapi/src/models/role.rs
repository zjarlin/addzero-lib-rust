// Generated from OpenAPI spec. Do not edit by hand.
//! `Role` DTO.

use serde::{Deserialize, Serialize};

/// Details about a role that can be assigned through the public Roles API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Always `role`.
    pub object: String,
    /// Identifier for the role.
    pub id: String,
    /// Unique name for the role.
    pub name: String,
    /// Optional description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Permissions granted by the role.
    pub permissions: Vec<String>,
    /// Resource type the role is bound to (for example `api.organization` or `api.project`).
    pub resource_type: String,
    /// Whether the role is predefined and managed by OpenAI.
    pub predefined_role: bool,
}
