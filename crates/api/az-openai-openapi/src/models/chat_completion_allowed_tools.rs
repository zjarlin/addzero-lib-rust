// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionAllowedTools` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Constrains the tools available to the model to a pre-defined set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionAllowedTools {
    /// Constrains the tools available to the model to a pre-defined set. `auto` allows the model to pick
    /// from among the allowed tools and generate a message. `required` requires the model to call one or
    /// more of the allowed tools.
    pub mode: String,
    /// A list of tool definitions that the model should be allowed to call. For the Chat Completions API,
    /// the list of tool definitions might look like: ```json [ { "type": "function", "function": { "name":
    /// "get_weather" } }, { "type": "function", "function": { "name": "get_time" } } ] ```
    pub tools: Vec<OpenAiJsonObject>,
}
