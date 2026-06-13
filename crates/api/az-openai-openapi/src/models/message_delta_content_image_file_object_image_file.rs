// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentImageFileObjectImageFile` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentImageFileObjectImageFile {
    /// The [File](/docs/api-reference/files) ID of the image in the message content. Set `purpose="vision"`
    /// when uploading the File if you need to later display the file content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// Specifies the detail level of the image if specified by the user. `low` uses fewer tokens, you can
    /// opt in to high resolution using `high`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}
