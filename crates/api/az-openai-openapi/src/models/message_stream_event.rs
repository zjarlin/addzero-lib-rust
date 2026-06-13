// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    MessageStreamEventObject,
    MessageStreamEventObject2,
    MessageStreamEventObject3,
    MessageStreamEventObject4,
    MessageStreamEventObject5,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageStreamEvent {
    Object(MessageStreamEventObject),
    Object2(MessageStreamEventObject2),
    Object3(MessageStreamEventObject3),
    Object4(MessageStreamEventObject4),
    Object5(MessageStreamEventObject5),
}
