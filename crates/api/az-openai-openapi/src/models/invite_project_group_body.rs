// Generated from OpenAPI spec. Do not edit by hand.
//! `InviteProjectGroupBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for granting a group access to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteProjectGroupBody {
    /// Identifier of the group to add to the project.
    pub group_id: String,
    /// Identifier of the project role to grant to the group.
    pub role: String,
}
