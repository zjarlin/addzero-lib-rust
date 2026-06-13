// Generated from OpenAPI spec. Do not edit by hand.
//! `Conversation2` DTO.

use serde::{Deserialize, Serialize};

/// The conversation that this response belonged to. Input items and output items from this response
/// were automatically added to this conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation2 {
    /// The unique ID of the conversation that this response was associated with.
    pub id: String,
}
