// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CustomToolParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolParamFormat,
};

/// A custom tool that processes input using a specified format. Learn more about [custom
/// tools](/docs/guides/function-calling#custom-tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomToolParam {
    /// The type of the custom tool. Always `custom`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The name of the custom tool, used to identify it in tool calls.
    pub name: String,
    /// Optional description of the custom tool, used to provide more context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The input format for the custom tool. Default is unconstrained text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<CustomToolParamFormat>,
    /// Whether this tool should be deferred and discovered via tool search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}
