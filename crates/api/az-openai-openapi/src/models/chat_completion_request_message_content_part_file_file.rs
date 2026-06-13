// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionRequestMessageContentPartFileFile` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestMessageContentPartFileFile {
    /// The name of the file, used when passing the file to the model as a string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// The base64 encoded file data, used when passing the file to the model as a string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// The ID of an uploaded file to use as input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
