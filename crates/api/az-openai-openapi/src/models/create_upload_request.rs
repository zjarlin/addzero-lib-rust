// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateUploadRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileExpirationAfter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUploadRequest {
    /// The name of the file to upload.
    pub filename: String,
    /// The intended purpose of the uploaded file. See the [documentation on File purposes](/docs/api-
    /// reference/files/create#files-create-purpose).
    pub purpose: String,
    /// The number of bytes in the file you are uploading.
    pub bytes: i32,
    /// The MIME type of the file. This must fall within the supported MIME types for your file purpose. See
    /// the supported MIME types for assistants and vision.
    pub mime_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<FileExpirationAfter>,
}
