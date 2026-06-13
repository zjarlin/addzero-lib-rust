// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AssistantsNamedToolChoiceFunction` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantsNamedToolChoiceFunction {
    /// The name of the function to call.
    pub name: String,
}
