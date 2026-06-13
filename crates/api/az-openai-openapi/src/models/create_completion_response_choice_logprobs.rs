// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateCompletionResponseChoiceLogprobs` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompletionResponseChoiceLogprobs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_offset: Option<Vec<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_logprobs: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<Vec<std::collections::BTreeMap<String, f64>>>,
}
