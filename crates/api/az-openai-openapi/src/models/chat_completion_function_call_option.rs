// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionFunctionCallOption` DTO.

use serde::{Deserialize, Serialize};

/// Specifying a particular function via `{"name": "my_function"}` forces the model to call that
/// function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionFunctionCallOption {
    /// The name of the function to call.
    pub name: String,
}
