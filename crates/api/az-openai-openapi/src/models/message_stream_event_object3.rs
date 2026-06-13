// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageStreamEventObject3` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaObject,
};

/// Occurs when parts of a [Message](/docs/api-reference/messages/object) are being streamed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStreamEventObject3 {
    pub event: String,
    pub data: MessageDeltaObject,
}
