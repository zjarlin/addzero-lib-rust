// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneReinforcementHyperparametersBatchSize` DTO.

use serde::{Deserialize, Serialize};

/// Number of examples in each batch. A larger batch size means that model parameters are updated less
/// frequently, but with lower variance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersBatchSize {
    Auto(String),
    Integer(i32),
}
