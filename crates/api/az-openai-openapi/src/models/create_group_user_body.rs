// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateGroupUserBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for adding a user to a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupUserBody {
    /// Identifier of the user to add to the group.
    pub user_id: String,
}
