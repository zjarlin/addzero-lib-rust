// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsFileSearchRankingOptionsObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileSearchRanker,
};

/// The ranking options for the file search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsFileSearchRankingOptionsObject {
    pub ranker: FileSearchRanker,
    /// The score threshold for the file search. All values must be a floating point number between 0 and 1.
    pub score_threshold: f64,
}
