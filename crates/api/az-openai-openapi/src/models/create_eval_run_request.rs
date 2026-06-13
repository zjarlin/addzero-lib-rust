// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalRunRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalRunRequestDataSource,
    Metadata,
};

/// CreateEvalRunRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvalRunRequest {
    /// The name of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Details about the run's data source.
    pub data_source: CreateEvalRunRequestDataSource,
}
