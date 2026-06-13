// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuningJobHyperparametersBatchSize` DTO.

use serde::{Deserialize, Serialize};

/// Number of examples in each batch. A larger batch size means that model parameters are updated less
/// frequently, but with lower variance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuningJobHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
