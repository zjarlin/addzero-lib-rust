// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage {
    /// The [file](/docs/api-reference/files) ID of the image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}
