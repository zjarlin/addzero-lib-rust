// Generated from OpenAPI spec. Do not edit by hand.
//! `RunToolCallObjectFunction` DTO.

use serde::{Deserialize, Serialize};

/// The function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunToolCallObjectFunction {
    /// The name of the function.
    pub name: String,
    /// The arguments that the model expects you to pass to the function.
    pub arguments: String,
}
