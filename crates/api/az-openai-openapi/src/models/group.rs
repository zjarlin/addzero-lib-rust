// Generated from OpenAPI spec. Do not edit by hand.
//! `Group` DTO.

use serde::{Deserialize, Serialize};

/// Summary information about a group returned in role assignment responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Always `group`.
    pub object: String,
    /// Identifier for the group.
    pub id: String,
    /// Display name of the group.
    pub name: String,
    /// Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    /// Whether the group is managed through SCIM.
    pub scim_managed: bool,
}
