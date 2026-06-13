// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `LocalEnvironmentResource` DTO.

use serde::{Deserialize, Serialize};

/// Represents the use of a local environment to perform shell actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEnvironmentResource {
    /// The environment type. Always `local`.
    #[serde(rename = "type")]
    pub type_value: String,
}
