// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageStreamEventObject2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObject,
};

/// Occurs when a [message](/docs/api-reference/messages/object) moves to an `in_progress` state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamEventObject2 {
    pub event: String,
    pub data: MessageObject,
}
