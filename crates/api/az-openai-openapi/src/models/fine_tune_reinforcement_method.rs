// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneReinforcementMethod` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneReinforcementHyperparameters,
    FineTuneReinforcementMethodGrader,
};

/// Configuration for the reinforcement fine-tuning method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneReinforcementMethod {
    /// The grader used for the fine-tuning job.
    pub grader: FineTuneReinforcementMethodGrader,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<FineTuneReinforcementHyperparameters>,
}
