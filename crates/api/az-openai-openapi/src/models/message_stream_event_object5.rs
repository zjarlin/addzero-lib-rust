// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageStreamEventObject5` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageObject,
};

/// Occurs when a [message](/docs/api-reference/messages/object) ends before it is completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamEventObject5 {
    pub event: String,
    pub data: MessageObject,
}
