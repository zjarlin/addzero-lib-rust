// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ImageEditStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ImageEditCompletedEvent,
    ImageEditPartialImageEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageEditStreamEvent {
    ImageEditPartialImageEvent(ImageEditPartialImageEvent),
    ImageEditCompletedEvent(ImageEditCompletedEvent),
}
