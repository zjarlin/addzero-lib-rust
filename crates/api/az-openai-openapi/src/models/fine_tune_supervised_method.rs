// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneSupervisedMethod` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneSupervisedHyperparameters,
};

/// Configuration for the supervised fine-tuning method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneSupervisedMethod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<FineTuneSupervisedHyperparameters>,
}
