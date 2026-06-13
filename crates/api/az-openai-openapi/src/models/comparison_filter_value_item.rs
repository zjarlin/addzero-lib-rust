// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComparisonFilterValueItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValueItem {
    String(String),
    Number(f64),
}
