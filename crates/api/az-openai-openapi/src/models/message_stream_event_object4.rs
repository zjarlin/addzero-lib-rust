// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageStreamEventObject4` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObject,
};

/// Occurs when a [message](/docs/api-reference/messages/object) is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamEventObject4 {
    pub event: String,
    pub data: MessageObject,
}
