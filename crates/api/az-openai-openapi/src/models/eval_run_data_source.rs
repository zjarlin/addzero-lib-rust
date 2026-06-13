// Generated from OpenAPI spec. Do not edit by hand.
//! `EvalRunDataSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCompletionsRunDataSource,
    CreateEvalJsonlRunDataSource,
    CreateEvalResponsesRunDataSource,
};

/// Information about the run's data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvalRunDataSource {
    CreateEvalJsonlRunDataSource(CreateEvalJsonlRunDataSource),
    CreateEvalCompletionsRunDataSource(CreateEvalCompletionsRunDataSource),
    CreateEvalResponsesRunDataSource(CreateEvalResponsesRunDataSource),
}
