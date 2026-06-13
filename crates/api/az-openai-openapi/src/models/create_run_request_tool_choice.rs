// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateRunRequestToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunRequestToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
