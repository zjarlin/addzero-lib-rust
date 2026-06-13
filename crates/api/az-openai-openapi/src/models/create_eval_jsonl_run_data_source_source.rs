// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalJsonlRunDataSourceSource` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalJsonlFileContentSource,
    EvalJsonlFileIdSource,
};

/// Determines what populates the `item` namespace in the data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalJsonlRunDataSourceSource {
    EvalJsonlFileContentSource(EvalJsonlFileContentSource),
    EvalJsonlFileIdSource(EvalJsonlFileIdSource),
}
