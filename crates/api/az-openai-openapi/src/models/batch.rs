// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Batch` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    BatchErrors,
    BatchRequestCounts,
    BatchUsage,
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    /// The object type, which is always `batch`.
    pub object: String,
    /// The OpenAI API endpoint used by the batch.
    pub endpoint: String,
    /// Model ID used to process the batch, like `gpt-5-2025-08-07`. OpenAI offers a wide range of models
    /// with different capabilities, performance characteristics, and price points. Refer to the [model
    /// guide](/docs/models) to browse and compare available models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<BatchErrors>,
    /// The ID of the input file for the batch.
    pub input_file_id: String,
    /// The time frame within which the batch should be processed.
    pub completion_window: String,
    /// The current status of the batch.
    pub status: String,
    /// The ID of the file containing the outputs of successfully executed requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file_id: Option<String>,
    /// The ID of the file containing the outputs of requests with errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_file_id: Option<String>,
    /// The Unix timestamp (in seconds) for when the batch was created.
    pub created_at: i64,
    /// The Unix timestamp (in seconds) for when the batch started processing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_progress_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch will expire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch started finalizing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finalizing_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch was completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch expired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch started cancelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelling_at: Option<i64>,
    /// The Unix timestamp (in seconds) for when the batch was cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<i64>,
    /// The request counts for different statuses within the batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_counts: Option<BatchRequestCounts>,
    /// Represents token usage details including input tokens, output tokens, a breakdown of output tokens,
    /// and the total tokens used. Only populated on batches created after September 7, 2025.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<BatchUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
