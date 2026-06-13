// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaContentTextAnnotationsFilePathObjectFilePath` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFilePathObjectFilePath {
    /// The ID of the file that was generated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
