// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalItemContentOutputText` DTO.

use serde::{Deserialize, Serialize};

/// A text output from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalItemContentOutputText {
    /// The type of the output text. Always `output_text`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The text output from the model.
    pub text: String,
}
