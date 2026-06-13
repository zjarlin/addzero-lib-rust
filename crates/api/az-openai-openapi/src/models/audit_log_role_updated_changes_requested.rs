// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogRoleUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// The payload used to update the role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogRoleUpdatedChangesRequested {
    /// The updated role name, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
    /// The resource the role is scoped to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// The type of resource the role belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// The permissions added to the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions_added: Option<Vec<String>>,
    /// The permissions removed from the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions_removed: Option<Vec<String>>,
    /// The updated role description, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Additional metadata stored on the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
}
