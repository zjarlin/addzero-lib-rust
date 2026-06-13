// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageContentImageFileObjectImageFile` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentImageFileObjectImageFile {
    /// The [File](/docs/api-reference/files) ID of the image in the message content. Set `purpose="vision"`
    /// when uploading the File if you need to later display the file content.
    pub file_id: String,
    /// Specifies the detail level of the image if specified by the user. `low` uses fewer tokens, you can
    /// opt in to high resolution using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
