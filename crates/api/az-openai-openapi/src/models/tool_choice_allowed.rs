// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ToolChoiceAllowed` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceAllowed {
    /// Allowed tool configuration type. Always `allowed_tools`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Constrains the tools available to the model to a pre-defined set. `auto` allows the model to pick
    /// from among the allowed tools and generate a message. `required` requires the model to call one or
    /// more of the allowed tools.
    pub mode: String,
    /// A list of tool definitions that the model should be allowed to call. For the Responses API, the list
    /// of tool definitions might look like: ```json [ { "type": "function", "name": "get_weather" }, {
    /// "type": "mcp", "server_label": "deepwiki" }, { "type": "image_generation" } ] ```
    pub tools: Vec<OpenAiJsonObject>,
}
