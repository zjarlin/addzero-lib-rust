// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerToolCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComputerAction,
    ComputerActionList,
    ComputerCallSafetyCheckParam,
};

/// A tool call to a computer use tool. See the [computer use guide](/docs/guides/tools-computer-use)
/// for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerToolCall {
    /// The type of the computer call. Always `computer_call`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The unique ID of the computer call.
    pub id: String,
    /// An identifier used when responding to the tool call with output.
    pub call_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ComputerAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<ComputerActionList>,
    /// The pending safety checks for the computer call.
    pub pending_safety_checks: Vec<ComputerCallSafetyCheckParam>,
    /// The status of the item. One of `in_progress`, `completed`, or `incomplete`. Populated when items are
    /// returned via API.
    pub status: String,
}
