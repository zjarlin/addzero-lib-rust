// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `VectorStoreSearchRequestQuery` DTO.

use serde::{Deserialize, Serialize};

/// A query string for a search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorStoreSearchRequestQuery {
    String(String),
    Array(Vec<String>),
}
