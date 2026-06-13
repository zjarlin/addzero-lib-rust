// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookResponseFailedData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseFailedData {
    /// The unique ID of the model response.
    pub id: String,
}
