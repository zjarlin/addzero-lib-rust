// Generated from OpenAPI spec. Do not edit by hand.
//! `GroupUser` DTO.

use serde::{Deserialize, Serialize};

/// Represents an individual user returned when inspecting group membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUser {
    /// The identifier, which can be referenced in API endpoints
    pub id: String,
    /// The name of the user.
    pub name: String,
    /// The email address of the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}
