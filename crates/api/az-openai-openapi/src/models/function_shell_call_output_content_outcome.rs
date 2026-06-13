// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionShellCallOutputContentOutcome` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FunctionShellCallOutputExitOutcome,
    FunctionShellCallOutputTimeoutOutcome,
};

/// Represents either an exit outcome (with an exit code) or a timeout outcome for a shell call output
/// chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionShellCallOutputContentOutcome {
    FunctionShellCallOutputTimeoutOutcome(FunctionShellCallOutputTimeoutOutcome),
    FunctionShellCallOutputExitOutcome(FunctionShellCallOutputExitOutcome),
}
