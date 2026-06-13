// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `HybridSearchOptions` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchOptions {
    /// The weight of the embedding in the reciprocal ranking fusion.
    pub embedding_weight: f64,
    /// The weight of the text in the reciprocal ranking fusion.
    pub text_weight: f64,
}
