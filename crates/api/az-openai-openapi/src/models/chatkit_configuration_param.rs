// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatkitConfigurationParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AutomaticThreadTitlingParam,
    FileUploadParam,
    HistoryParam,
};

/// Optional per-session configuration settings for ChatKit behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatkitConfigurationParam {
    /// Configuration for automatic thread titling. When omitted, automatic thread titling is enabled by
    /// default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_thread_titling: Option<AutomaticThreadTitlingParam>,
    /// Configuration for upload enablement and limits. When omitted, uploads are disabled by default
    /// (max_files 10, max_file_size 512 MB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_upload: Option<FileUploadParam>,
    /// Configuration for chat history retention. When omitted, history is enabled by default with no limit
    /// on recent_threads (null).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryParam>,
}
