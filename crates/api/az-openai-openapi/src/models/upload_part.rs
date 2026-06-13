// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UploadPart` DTO.

use serde::{Deserialize, Serialize};

/// The upload Part represents a chunk of bytes we can add to an Upload object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPart {
    /// The upload Part unique identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the Part was created.
    pub created_at: i64,
    /// The ID of the Upload object that this Part was added to.
    pub upload_id: String,
    /// The object type, which is always `upload.part`.
    pub object: String,
}
