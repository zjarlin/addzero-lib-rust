// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CodeInterpreterFileOutputFile` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterFileOutputFile {
    /// The MIME type of the file.
    pub mime_type: String,
    /// The ID of the file.
    pub file_id: String,
}
