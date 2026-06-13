// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `EvalDataSourceConfig` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EvalCustomDataSourceConfig,
    EvalLogsDataSourceConfig,
    EvalStoredCompletionsDataSourceConfig,
};

/// Configuration of data sources used in runs of the evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvalDataSourceConfig {
    EvalCustomDataSourceConfig(EvalCustomDataSourceConfig),
    EvalLogsDataSourceConfig(EvalLogsDataSourceConfig),
    EvalStoredCompletionsDataSourceConfig(EvalStoredCompletionsDataSourceConfig),
}
