// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OpenAIFile` DTO.

use serde::{Deserialize, Serialize};

/// The `File` object represents a document that has been uploaded to OpenAI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFile {
    /// The file identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The size of the file, in bytes.
    pub bytes: i32,
    /// The Unix timestamp (in seconds) for when the file was created.
    pub created_at: i64,
    /// The Unix timestamp (in seconds) for when the file will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// The name of the file.
    pub filename: String,
    /// The object type, which is always `file`.
    pub object: String,
    /// The intended purpose of the file. Supported values are `assistants`, `assistants_output`, `batch`,
    /// `batch_output`, `fine-tune`, `fine-tune-results`, `vision`, and `user_data`.
    pub purpose: String,
    /// Deprecated. The current status of the file, which can be either `uploaded`, `processed`, or `error`.
    pub status: String,
    /// Deprecated. For details on why a fine-tuning training file failed validation, see the `error` field
    /// on `fine_tuning.job`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,
}
