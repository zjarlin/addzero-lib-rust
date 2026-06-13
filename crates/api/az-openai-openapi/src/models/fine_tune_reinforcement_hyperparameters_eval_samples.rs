// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneReinforcementHyperparametersEvalSamples` DTO.

use serde::{Deserialize, Serialize};

/// Number of evaluation samples to generate per training step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersEvalSamples {
    Auto(String),
    Integer(i32),
}
