// Generated from OpenAPI spec. Do not edit by hand.
//! `WebhookResponseCancelledData` DTO.

use serde::{Deserialize, Serialize};

/// Event data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponseCancelledData {
    /// The unique ID of the model response.
    pub id: String,
}
