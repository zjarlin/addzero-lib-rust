// Generated from OpenAPI spec. Do not edit by hand.
//! `Model` DTO.

use serde::{Deserialize, Serialize};

/// Describes an OpenAI model offering that can be used with the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// The model identifier, which can be referenced in the API endpoints.
    pub id: String,
    /// The Unix timestamp (in seconds) when the model was created.
    pub created: i64,
    /// The object type, which is always "model".
    pub object: String,
    /// The organization that owns the model.
    pub owned_by: String,
}
