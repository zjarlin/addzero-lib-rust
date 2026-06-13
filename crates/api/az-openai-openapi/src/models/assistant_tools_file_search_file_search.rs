// Generated from OpenAPI spec. Do not edit by hand.
//! `AssistantToolsFileSearchFileSearch` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FileSearchRankingOptions,
};

/// Overrides for the file search tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolsFileSearchFileSearch {
    /// The maximum number of results the file search tool should output. The default is 20 for `gpt-4*`
    /// models and 5 for `gpt-3.5-turbo`. This number should be between 1 and 50 inclusive. Note that the
    /// file search tool may output fewer than `max_num_results` results. See the [file search tool
    /// documentation](/docs/assistants/tools/file-search#customizing-file-search-settings) for more
    /// information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<FileSearchRankingOptions>,
}
