// Generated from OpenAPI spec. Do not edit by hand.
//! `Reasoning` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ReasoningEffort,
};

/// **gpt-5 and o-series models only** Configuration options for [reasoning
/// models](https://platform.openai.com/docs/guides/reasoning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reasoning {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_summary: Option<String>,
}
