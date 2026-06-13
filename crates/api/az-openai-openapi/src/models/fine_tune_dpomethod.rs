// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneDPOMethod` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneDPOHyperparameters,
};

/// Configuration for the DPO fine-tuning method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneDPOMethod {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<FineTuneDPOHyperparameters>,
}
