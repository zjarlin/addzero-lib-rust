// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaStepDetailsToolCallsCodeOutputImageObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage,
};

/// Code interpreter image output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaStepDetailsToolCallsCodeOutputImageObject {
    /// The index of the output in the outputs array.
    pub index: i32,
    /// Always `image`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<RunStepDeltaStepDetailsToolCallsCodeOutputImageObjectImage>,
}
