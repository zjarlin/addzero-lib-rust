// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CompleteUploadRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteUploadRequest {
    /// The ordered list of Part IDs.
    pub part_ids: Vec<String>,
    /// The optional md5 checksum for the file contents to verify if the bytes uploaded matches what you
    /// expect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}
