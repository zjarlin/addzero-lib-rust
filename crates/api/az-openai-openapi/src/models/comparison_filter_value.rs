// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComparisonFilterValue` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ComparisonFilterValueArrayItem,
};

/// The value to compare against the attribute key; supports string, number, or boolean types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ComparisonFilterValueArrayItem>),
}
