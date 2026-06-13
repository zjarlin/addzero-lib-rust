// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentImageFileObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaContentImageFileObjectImageFile,
};

/// References an image [File](/docs/api-reference/files) in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentImageFileObject {
    /// The index of the content part in the message.
    pub index: i32,
    /// Always `image_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_file: Option<MessageDeltaContentImageFileObjectImageFile>,
}
