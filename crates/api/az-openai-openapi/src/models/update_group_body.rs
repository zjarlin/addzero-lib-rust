// Generated from OpenAPI spec. Do not edit by hand.
//! `UpdateGroupBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for updating the details of an existing group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupBody {
    /// New display name for the group.
    pub name: String,
}
