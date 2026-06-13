// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatCompletionNamedToolChoiceFunction` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionNamedToolChoiceFunction {
    /// The name of the function to call.
    pub name: String,
}
