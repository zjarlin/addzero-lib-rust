// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneSupervisedHyperparametersLearningRateMultiplier` DTO.

use serde::{Deserialize, Serialize};

/// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneSupervisedHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
