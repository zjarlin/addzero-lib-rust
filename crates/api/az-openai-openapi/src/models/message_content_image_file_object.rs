// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageContentImageFileObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageContentImageFileObjectImageFile,
};

/// References an image [File](/docs/api-reference/files) in the content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentImageFileObject {
    /// Always `image_file`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub image_file: MessageContentImageFileObjectImageFile,
}
