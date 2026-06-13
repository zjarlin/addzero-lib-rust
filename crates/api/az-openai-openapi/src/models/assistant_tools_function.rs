// Generated from OpenAPI spec. Do not edit by hand.
//! `AssistantToolsFunction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionObject,
};

/// Function tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolsFunction {
    /// The type of tool being defined: `function`
    #[serde(rename = "type")]
    pub type_value: String,
    pub function: FunctionObject,
}
