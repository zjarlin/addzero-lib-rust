// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepDeltaObjectDelta` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaObjectDeltaStepDetails,
};

/// The delta containing the fields that have changed on the run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaObjectDelta {
    /// The details of the run step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_details: Option<RunStepDeltaObjectDeltaStepDetails>,
}
