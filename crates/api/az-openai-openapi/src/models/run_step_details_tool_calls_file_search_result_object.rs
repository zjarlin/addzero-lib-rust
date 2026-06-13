// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDetailsToolCallsFileSearchResultObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsFileSearchResultObjectContentItem,
};

/// A result instance of the file search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchResultObject {
    /// The ID of the file that result was found in.
    pub file_id: String,
    /// The name of the file that result was found in.
    pub file_name: String,
    /// The score of the result. All values must be a floating point number between 0 and 1.
    pub score: f64,
    /// The content of the result that was found. The content is only included if requested via the include
    /// query parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<RunStepDetailsToolCallsFileSearchResultObjectContentItem>>,
}
