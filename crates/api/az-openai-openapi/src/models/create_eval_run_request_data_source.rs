// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalRunRequestDataSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCompletionsRunDataSource,
    CreateEvalJsonlRunDataSource,
    CreateEvalResponsesRunDataSource,
};

/// Details about the run's data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRunRequestDataSource {
    CreateEvalJsonlRunDataSource(CreateEvalJsonlRunDataSource),
    CreateEvalCompletionsRunDataSource(CreateEvalCompletionsRunDataSource),
    CreateEvalResponsesRunDataSource(CreateEvalResponsesRunDataSource),
}
