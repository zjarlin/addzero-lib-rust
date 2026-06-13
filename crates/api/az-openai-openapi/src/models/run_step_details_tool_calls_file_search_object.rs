// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsToolCallsFileSearchObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsFileSearchObjectFileSearch,
};

/// File search tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchObject {
    /// The ID of the tool call object.
    pub id: String,
    /// The type of tool call. This is always going to be `file_search` for this type of tool call.
    #[serde(rename = "type")]
    pub type_value: String,
    /// For now, this is always going to be an empty object.
    pub file_search: RunStepDetailsToolCallsFileSearchObjectFileSearch,
}
