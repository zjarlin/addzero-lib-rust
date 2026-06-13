// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AutomaticThreadTitlingParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls whether ChatKit automatically generates thread titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticThreadTitlingParam {
    /// Enable automatic thread title generation. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
