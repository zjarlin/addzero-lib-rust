// Generated from OpenAPI spec. Do not edit by hand.
//! `CreateFineTuningJobRequestHyperparametersLearningRateMultiplier` DTO.

use serde::{Deserialize, Serialize};

/// Scaling factor for the learning rate. A smaller learning rate may be useful to avoid overfitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateFineTuningJobRequestHyperparametersLearningRateMultiplier {
    Auto(String),
    Number(f64),
}
