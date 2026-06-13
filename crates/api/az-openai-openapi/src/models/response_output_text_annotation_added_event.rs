// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseOutputTextAnnotationAddedEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Emitted when an annotation is added to output text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOutputTextAnnotationAddedEvent {
    /// The type of the event. Always 'response.output_text.annotation.added'.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique identifier of the item to which the annotation is being added.
    pub item_id: String,
    /// The index of the output item in the response's output array.
    pub output_index: i32,
    /// The index of the content part within the output item.
    pub content_index: i32,
    /// The index of the annotation within the content part.
    pub annotation_index: i32,
    /// The sequence number of this event.
    pub sequence_number: i32,
    /// The annotation object being added. (See annotation schema for details.)
    pub annotation: OpenAiJsonObject,
}
