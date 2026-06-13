// Generated from OpenAPI spec. Do not edit by hand.
//! `OutputTextContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Annotation,
    LogProb,
};

/// A text output from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTextContent {
    /// The type of the output text. Always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text output from the model.
    pub text: String,
    /// The annotations of the text output.
    pub annotations: Vec<Annotation>,
    pub logprobs: Vec<LogProb>,
}
