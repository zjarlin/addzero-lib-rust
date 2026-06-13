// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `UpdateEvalRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvalRequest {
    /// Rename the evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}
