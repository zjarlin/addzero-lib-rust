// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateChatCompletionRequestFunctionCall` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatCompletionFunctionCallOption,
};

/// Deprecated in favor of `tool_choice`. Controls which (if any) function is called by the model.
/// `none` means the model will not call a function and instead generates a message. `auto` means the
/// model can pick between generating a message or calling a function. Specifying a particular function
/// via `{"name": "my_function"}` forces the model to call that function. `none` is the default when no
/// functions are present. `auto` is the default if functions are present.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateChatCompletionRequestFunctionCall {
    String(String),
    ChatCompletionFunctionCallOption(ChatCompletionFunctionCallOption),
}
