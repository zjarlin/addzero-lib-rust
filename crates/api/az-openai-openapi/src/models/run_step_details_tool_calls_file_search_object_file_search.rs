// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsFileSearchObjectFileSearch` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsFileSearchRankingOptionsObject,
    RunStepDetailsToolCallsFileSearchResultObject,
};

/// For now, this is always going to be an empty object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchObjectFileSearch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<RunStepDetailsToolCallsFileSearchRankingOptionsObject>,
    /// The results of the file search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<RunStepDetailsToolCallsFileSearchResultObject>>,
}
