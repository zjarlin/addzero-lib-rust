// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `Embedding` DTO.

use serde::{Deserialize, Serialize};

/// Represents an embedding vector returned by embedding endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    /// The index of the embedding in the list of embeddings.
    pub index: i32,
    /// The embedding vector, which is a list of floats. The length of vector depends on the model as listed
    /// in the [embedding guide](/docs/guides/embeddings).
    pub embedding: Vec<f32>,
    /// The object type, which is always "embedding".
    pub object: String,
}
