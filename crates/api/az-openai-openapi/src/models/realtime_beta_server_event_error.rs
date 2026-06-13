// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeBetaServerEventError` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeBetaServerEventErrorError,
};

/// Returned when an error occurs, which could be a client problem or a server problem. Most errors are
/// recoverable and the session will stay open, we recommend to implementors to monitor and log error
/// messages by default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeBetaServerEventError {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `error`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Details of the error.
    pub error: RealtimeBetaServerEventErrorError,
}
