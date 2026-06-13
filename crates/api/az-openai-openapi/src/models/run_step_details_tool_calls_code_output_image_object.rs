// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDetailsToolCallsCodeOutputImageObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDetailsToolCallsCodeOutputImageObjectImage,
};

/// Code Interpreter image output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDetailsToolCallsCodeOutputImageObject {
    /// Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub image: RunStepDetailsToolCallsCodeOutputImageObjectImage,
}
