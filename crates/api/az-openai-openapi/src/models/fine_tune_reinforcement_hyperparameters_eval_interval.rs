// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneReinforcementHyperparametersEvalInterval` DTO.

use serde::{Deserialize, Serialize};

/// The number of training steps between evaluation runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersEvalInterval {
    Auto(String),
    Integer(i32),
}
