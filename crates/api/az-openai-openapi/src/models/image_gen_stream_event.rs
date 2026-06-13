// Generated from OpenAPI spec. Do not edit by hand.
//! `ImageGenStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageGenCompletedEvent,
    ImageGenPartialImageEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageGenStreamEvent {
    ImageGenPartialImageEvent(ImageGenPartialImageEvent),
    ImageGenCompletedEvent(ImageGenCompletedEvent),
}
