// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateBatchRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    BatchFileExpirationAfter,
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    /// The ID of an uploaded file that contains requests for the new batch. See [upload file](/docs/api-
    /// reference/files/create) for how to upload a file. Your input file must be formatted as a [JSONL
    /// file](/docs/api-reference/batch/request-input), and must be uploaded with the purpose `batch`. The
    /// file can contain up to 50,000 requests, and can be up to 200 MB in size.
    pub input_file_id: String,
    /// The endpoint to be used for all requests in the batch. Currently `/v1/responses`,
    /// `/v1/chat/completions`, `/v1/embeddings`, `/v1/completions`, `/v1/moderations`,
    /// `/v1/images/generations`, `/v1/images/edits`, and `/v1/videos` are supported. Note that
    /// `/v1/embeddings` batches are also restricted to a maximum of 50,000 embedding inputs across all
    /// requests in the batch.
    pub endpoint: String,
    /// The time frame within which the batch should be processed. Currently only `24h` is supported.
    pub completion_window: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_expires_after: Option<BatchFileExpirationAfter>,
}
