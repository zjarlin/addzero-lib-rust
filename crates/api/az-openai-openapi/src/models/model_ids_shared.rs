// Generated from OpenAPI spec. Do not edit by hand.
//! `ModelIdsShared` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelIdsShared {
    String(String),
    String2(String),
}
