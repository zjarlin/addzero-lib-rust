// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `FunctionShellCallOutputOutcomeParam` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionShellCallOutputExitOutcomeParam,
    FunctionShellCallOutputTimeoutOutcomeParam,
};

/// The exit or timeout outcome associated with this shell call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallOutputOutcomeParam {
    FunctionShellCallOutputTimeoutOutcomeParam(FunctionShellCallOutputTimeoutOutcomeParam),
    FunctionShellCallOutputExitOutcomeParam(FunctionShellCallOutputExitOutcomeParam),
}
