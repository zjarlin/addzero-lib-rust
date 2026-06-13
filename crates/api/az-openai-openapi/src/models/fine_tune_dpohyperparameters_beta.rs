// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FineTuneDPOHyperparametersBeta` DTO.

use serde::{Deserialize, Serialize};

/// The beta value for the DPO method. A higher beta value will increase the weight of the penalty
/// between the policy and reference model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineTuneDPOHyperparametersBeta {
    Auto(String),
    Number(f64),
}
