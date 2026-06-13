// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateEvalResponsesRunDataSourceSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalJsonlFileContentSource,
    EvalJsonlFileIdSource,
    EvalResponsesSource,
};

/// Determines what populates the `item` namespace in this run's data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalResponsesRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
    EvalResponsesSource(EvalResponsesSource),
}
