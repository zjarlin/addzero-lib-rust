// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `GroupDeletedResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after deleting a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDeletedResource {
    /// Always `group.deleted`.
    pub object: String,
    /// Identifier of the deleted group.
    pub id: String,
    /// Whether the group was deleted.
    pub deleted: bool,
}
