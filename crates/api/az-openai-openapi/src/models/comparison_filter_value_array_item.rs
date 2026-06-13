// Generated from OpenAPI spec. Do not edit by hand.
//! `ComparisonFilterValueArrayItem` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComparisonFilterValueArrayItem {
    String(String),
    Number(f64),
}
