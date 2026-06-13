// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerToolCallOutputResource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComputerCallSafetyCheckParam,
    ComputerScreenshotImage,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerToolCallOutputResource {
    /// The type of the computer tool call output. Always `computer_call_output`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the computer tool call output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The ID of the computer tool call that produced the output.
    pub call_id: String,
    /// The safety checks reported by the API that have been acknowledged by the developer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_safety_checks: Option<Vec<ComputerCallSafetyCheckParam>>,
    pub output: ComputerScreenshotImage,
    /// The status of the message input. One of `in_progress`, `completed`, or `incomplete`. Populated when
    /// input items are returned via API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The identifier of the actor that created the item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}
