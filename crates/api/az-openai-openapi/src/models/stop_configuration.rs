// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `StopConfiguration` DTO.

use serde::{Deserialize, Serialize};

/// Not supported with latest reasoning models `o3` and `o4-mini`. Up to 4 sequences where the API will
/// stop generating further tokens. The returned text will not contain the stop sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopConfiguration {
    String(String),
    Array(Vec<String>),
}
