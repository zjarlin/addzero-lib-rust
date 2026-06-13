// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneDPOHyperparametersNEpochs` DTO.

use serde::{Deserialize, Serialize};

/// The number of epochs to train the model for. An epoch refers to one full cycle through the training
/// dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneDPOHyperparametersNEpochs {
    Auto(String),
    Integer(i32),
}
