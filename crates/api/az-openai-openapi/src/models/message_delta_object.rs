// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageDeltaObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageDeltaObjectDelta,
};

/// Represents a message delta i.e. any changed fields on a message during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaObject {
    /// The identifier of the message, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread.message.delta`.
    pub object: String,
    /// The delta containing the fields that have changed on the Message.
    pub delta: MessageDeltaObjectDelta,
}
