// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneReinforcementHyperparametersNEpochs` DTO.

use serde::{Deserialize, Serialize};

/// The number of epochs to train the model for. An epoch refers to one full cycle through the training
/// dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneReinforcementHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
