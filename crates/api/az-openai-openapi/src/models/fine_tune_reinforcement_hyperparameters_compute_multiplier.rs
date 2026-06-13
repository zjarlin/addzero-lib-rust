// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneReinforcementHyperparametersComputeMultiplier` DTO.

use serde::{Deserialize, Serialize};

/// Multiplier on amount of compute used for exploring search space during training.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersComputeMultiplier {
    Auto(String),
    Number(f64),
}
