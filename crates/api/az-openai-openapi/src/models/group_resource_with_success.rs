// Generated from OpenAPI spec. Do not edit by hand.
//! `GroupResourceWithSuccess` DTO.

use serde::{Deserialize, Serialize};

/// Response returned after updating a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResourceWithSuccess {
    /// Identifier for the group.
    pub id: String,
    /// Updated display name for the group.
    pub name: String,
    /// Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    /// Whether the group is managed through SCIM and controlled by your identity provider.
    pub is_scim_managed: bool,
}
