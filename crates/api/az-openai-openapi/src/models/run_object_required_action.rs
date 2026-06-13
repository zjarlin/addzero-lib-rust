// Generated from OpenAPI spec. Do not edit by hand.
//! `RunObjectRequiredAction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObjectRequiredActionSubmitToolOutputs,
};

/// Details on the action required to continue the run. Will be `null` if no action is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunObjectRequiredAction {
    /// For now, this is always `submit_tool_outputs`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Details on the tool outputs needed for this run to continue.
    pub submit_tool_outputs: RunObjectRequiredActionSubmitToolOutputs,
}
