// Generated from OpenAPI spec. Do not edit by hand.
//! `ComputerCallOutputItemParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComputerCallSafetyCheckParam,
    ComputerScreenshotImage,
    FunctionCallItemStatus,
};

/// The output of a computer tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerCallOutputItemParam {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ID of the computer tool call that produced the output.
    pub call_id: String,
    /// The type of the computer tool call output. Always `computer_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub output: ComputerScreenshotImage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: Option<Vec<ComputerCallSafetyCheckParam>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<FunctionCallItemStatus>,
}
