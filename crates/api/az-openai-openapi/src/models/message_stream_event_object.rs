// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageStreamEventObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObject,
};

/// Occurs when a [message](/docs/api-reference/messages/object) is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamEventObject {
    pub event: String,
    pub data: MessageObject,
}
