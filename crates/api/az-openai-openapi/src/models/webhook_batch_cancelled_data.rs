// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookBatchCancelledData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookBatchCancelledData {
    /// The unique ID of the batch API request.
    pub id: String,
}
