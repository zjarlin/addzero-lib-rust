// Generated from OpenAPI spec. Do not edit by hand.
//! `CodeInterpreterOutputImage` DTO.

use serde::{Deserialize, Serialize};

/// The image output from the code interpreter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterOutputImage {
    /// The type of the output. Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The URL of the image output from the code interpreter.
    pub url: String,
}
