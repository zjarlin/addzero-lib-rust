// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Item` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

/// Content item used to generate a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
