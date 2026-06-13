// Generated from OpenAPI spec. Do not edit by hand.
//! `AssignedRoleDetails` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Detailed information about a role assignment entry returned when listing assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignedRoleDetails {
    /// Identifier for the role.
    pub id: String,
    /// Name of the role.
    pub name: String,
    /// Permissions associated with the role.
    pub permissions: Vec<String>,
    /// Resource type the role applies to.
    pub resource_type: String,
    /// Whether the role is predefined by OpenAI.
    pub predefined_role: bool,
    /// Description of the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// When the role was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// When the role was last updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    /// Identifier of the actor who created the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// User details for the actor that created the role, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_obj: Option<OpenAiJsonObject>,
    /// Arbitrary metadata stored on the role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiJsonObject>,
}
