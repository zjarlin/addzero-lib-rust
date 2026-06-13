// Generated from OpenAPI spec. Do not edit by hand.
//! `FineTuneMethod` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuneDPOMethod,
    FineTuneReinforcementMethod,
    FineTuneSupervisedMethod,
};

/// The method used for fine-tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneMethod {
    /// The type of method. Is either `supervised`, `dpo`, or `reinforcement`.
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervised: Option<FineTuneSupervisedMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpo: Option<FineTuneDPOMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reinforcement: Option<FineTuneReinforcementMethod>,
}
