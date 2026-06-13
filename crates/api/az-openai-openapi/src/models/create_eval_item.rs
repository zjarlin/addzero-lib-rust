// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalItem` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// A chat message that makes up the prompt or context. May include variable references to the `item`
/// namespace, ie {{item.name}}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalItem {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
