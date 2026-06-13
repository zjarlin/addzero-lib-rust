// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CreateEvalRequestDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CreateEvalCustomDataSourceConfig,
    CreateEvalLogsDataSourceConfig,
    CreateEvalStoredCompletionsDataSourceConfig,
};

/// The configuration for the data source used for the evaluation runs. Dictates the schema of the data
/// used in the evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateEvalRequestDataSourceConfig {
    CreateEvalCustomDataSourceConfig(CreateEvalCustomDataSourceConfig),
    CreateEvalLogsDataSourceConfig(CreateEvalLogsDataSourceConfig),
    CreateEvalStoredCompletionsDataSourceConfig(CreateEvalStoredCompletionsDataSourceConfig),
}
