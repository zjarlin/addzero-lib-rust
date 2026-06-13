// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedThreadResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after deleting a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedThreadResource {
    /// Identifier of the deleted thread.
    pub id: String,
    /// Type discriminator that is always `chatkit.thread.deleted`.
    pub object: String,
    /// Indicates that the thread has been deleted.
    pub deleted: bool,
}
