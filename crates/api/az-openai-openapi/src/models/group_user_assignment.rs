// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GroupUserAssignment` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after adding a user to a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUserAssignment {
    /// Always `group.user`.
    pub object: String,
    /// Identifier of the user that was added.
    pub user_id: String,
    /// Identifier of the group the user was added to.
    pub group_id: String,
}
