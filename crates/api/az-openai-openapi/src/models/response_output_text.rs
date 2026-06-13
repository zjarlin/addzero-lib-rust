// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseOutputText` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ResponseOutputTextAnnotation,
};

/// Assistant response text accompanied by optional annotations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOutputText {
    /// Type discriminator that is always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Assistant generated text.
    pub text: String,
    /// Ordered list of annotations attached to the response text.
    pub annotations: Vec<ResponseOutputTextAnnotation>,
}
