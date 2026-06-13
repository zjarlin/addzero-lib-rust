// Generated from OpenAPI spec. Do not edit by hand.
//! `DoneEvent` DTO.

use serde::{Deserialize, Serialize};

/// Occurs when a stream ends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneEvent {
    pub event: String,
    pub data: String,
}
