// Generated from OpenAPI spec. Do not edit by hand.
//! `DeletedVideoResource` DTO.

use serde::{Deserialize, Serialize};

/// Confirmation payload returned after deleting a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedVideoResource {
    /// The object type that signals the deletion response.
    pub object: String,
    /// Indicates that the video resource was deleted.
    pub deleted: bool,
    /// Identifier of the deleted video.
    pub id: String,
}
