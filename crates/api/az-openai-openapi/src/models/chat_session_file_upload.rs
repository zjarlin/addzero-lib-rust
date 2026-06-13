// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatSessionFileUpload` DTO.

use serde::{Deserialize, Serialize};

/// Upload permissions and limits applied to the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionFileUpload {
    /// Indicates if uploads are enabled for the session.
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: Option<i32>,
}
