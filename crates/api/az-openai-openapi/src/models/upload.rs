// Generated from OpenAPI spec. Do not edit by hand.
//! `Upload` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    UploadFile,
};

/// The Upload object can accept byte chunks in the form of Parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    /// The Upload unique identifier, which can be referenced in API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) for when the Upload was created.
    pub created_at: i64,
    /// The name of the file to be uploaded.
    pub filename: String,
    /// The intended number of bytes to be uploaded.
    pub bytes: i32,
    /// The intended purpose of the file. [Please refer here](/docs/api-reference/files/object#files/object-
    /// purpose) for acceptable values.
    pub purpose: String,
    /// The status of the Upload.
    pub status: String,
    /// The Unix timestamp (in seconds) for when the Upload will expire.
    pub expires_at: i64,
    /// The object type, which is always "upload".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<UploadFile>,
}
