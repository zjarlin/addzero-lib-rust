// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ResponseFormatTextPython` DTO.

use serde::{Deserialize, Serialize};

/// Configure the model to generate valid Python code. See the [custom grammars
/// guide](/docs/guides/custom-grammars) for more details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormatTextPython {
    /// The type of response format being defined. Always `python`.
    #[serde(rename = "type")]
    pub type_value: String,
}
