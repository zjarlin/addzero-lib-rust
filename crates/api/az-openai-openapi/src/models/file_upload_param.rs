// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FileUploadParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls whether users can upload files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadParam {
    /// Enable uploads for this session. Defaults to false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Maximum size in megabytes for each uploaded file. Defaults to 512 MB, which is the maximum allowable
    /// size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<i32>,
    /// Maximum number of files that can be uploaded to the session. Defaults to 10.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_files: Option<i32>,
}
