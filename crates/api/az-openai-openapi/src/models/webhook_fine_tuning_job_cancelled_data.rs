// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookFineTuningJobCancelledData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFineTuningJobCancelledData {
    /// The unique ID of the fine-tuning job.
    pub id: String,
}
