// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateGroupBody` DTO.

use serde::{Deserialize, Serialize};

/// Request payload for creating a new group in the organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupBody {
    /// Human readable name for the group.
    pub name: String,
}
