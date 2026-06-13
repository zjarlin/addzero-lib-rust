// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalCompletionsRunDataSourceSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalJsonlFileContentSource,
    EvalJsonlFileIdSource,
    EvalStoredCompletionsSource,
};

/// Determines what populates the `item` namespace in this run's data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalCompletionsRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
    EvalStoredCompletionsSource(EvalStoredCompletionsSource),
}
