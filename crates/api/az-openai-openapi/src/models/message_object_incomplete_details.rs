// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageObjectIncompleteDetails` DTO.

use serde::{Deserialize, Serialize};

/// On an incomplete message, details about why the message is incomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageObjectIncompleteDetails {
    /// The reason the message is incomplete.
    pub reason: String,
}
