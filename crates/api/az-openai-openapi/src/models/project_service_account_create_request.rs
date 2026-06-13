// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ProjectServiceAccountCreateRequest` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectServiceAccountCreateRequest {
    /// The name of the service account being created.
    pub name: String,
}
