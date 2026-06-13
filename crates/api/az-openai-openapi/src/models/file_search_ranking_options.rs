// Generated from OpenAPI spec. Do not edit by hand.
//! `FileSearchRankingOptions` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileSearchRanker,
};

/// The ranking options for the file search. If not specified, the file search tool will use the `auto`
/// ranker and a score_threshold of 0. See the [file search tool
/// documentation](/docs/assistants/tools/file-search#customizing-file-search-settings) for more
/// information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchRankingOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranker: Option<FileSearchRanker>,
    /// The score threshold for the file search. All values must be a floating point number between 0 and 1.
    pub score_threshold: f64,
}
