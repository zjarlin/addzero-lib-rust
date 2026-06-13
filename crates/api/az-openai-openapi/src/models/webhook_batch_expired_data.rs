// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookBatchExpiredData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchExpiredData {
    /// The unique ID of the batch API request.
    pub id: String,
}
