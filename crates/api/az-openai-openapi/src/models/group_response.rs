// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GroupResponse` DTO.

use serde::{Deserialize, Serialize};

/// Details about an organization group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    /// Identifier for the group.
    pub id: String,
    /// Display name of the group.
    pub name: String,
    /// Unix timestamp (in seconds) when the group was created.
    pub created_at: i64,
    /// Whether the group is managed through SCIM and controlled by your identity provider.
    pub is_scim_managed: bool,
    /// The type of the group.
    pub group_type: String,
}
