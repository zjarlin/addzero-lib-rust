// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStepStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepStreamEventObject,
    RunStepStreamEventObject2,
    RunStepStreamEventObject3,
    RunStepStreamEventObject4,
    RunStepStreamEventObject5,
    RunStepStreamEventObject6,
    RunStepStreamEventObject7,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStepStreamEvent {
    Object(RunStepStreamEventObject),
    Object2(RunStepStreamEventObject2),
    Object3(RunStepStreamEventObject3),
    Object4(RunStepStreamEventObject4),
    Object5(RunStepStreamEventObject5),
    Object6(RunStepStreamEventObject6),
    Object7(RunStepStreamEventObject7),
}
