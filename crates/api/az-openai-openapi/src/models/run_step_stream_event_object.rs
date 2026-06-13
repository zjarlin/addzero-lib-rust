// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepStreamEventObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepObject,
};

/// Occurs when a [run step](/docs/api-reference/run-steps/step-object) is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepStreamEventObject {
    pub event: String,
    pub data: RunStepObject,
}
