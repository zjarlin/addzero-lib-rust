// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateThreadAndRunRequestToolChoice` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThreadAndRunRequestToolChoice {
    #[serde(flatten)]
    pub value: OpenAiJsonObject,
}
