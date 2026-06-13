// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateFileRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiBinaryBody,
};

use crate::models::{
    FileExpirationAfter,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileRequest {
    /// The File object (not file name) to be uploaded.
    pub file: OpenAiBinaryBody,
    /// The intended purpose of the uploaded file. One of: - `assistants`: Used in the Assistants API -
    /// `batch`: Used in the Batch API - `fine-tune`: Used for fine-tuning - `vision`: Images used for
    /// vision fine-tuning - `user_data`: Flexible file type for any purpose - `evals`: Used for eval data
    /// sets
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<FileExpirationAfter>,
}
