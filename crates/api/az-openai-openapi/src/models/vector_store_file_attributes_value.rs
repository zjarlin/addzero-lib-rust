// Generated from OpenAPI spec. Do not edit by hand.
//! `VectorStoreFileAttributesValue` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorStoreFileAttributesValue {
    String(String),
    Number(f64),
    Boolean(bool),
}
