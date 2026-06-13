// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStepDeltaObject` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepDeltaObjectDelta,
};

/// Represents a run step delta i.e. any changed fields on a run step during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepDeltaObject {
    /// The identifier of the run step, which can be referenced in API endpoints.
    pub id: String,
    /// The object type, which is always `thread.run.step.delta`.
    pub object: String,
    /// The delta containing the fields that have changed on the run step.
    pub delta: RunStepDeltaObjectDelta,
}
