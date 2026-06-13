// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionObject,
};

/// A function tool that can be used to generate a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionTool {
    /// The type of the tool. Currently, only `function` is supported.
    #[serde(rename = "type")]
    pub type_value: String,
    pub function: FunctionObject,
}
