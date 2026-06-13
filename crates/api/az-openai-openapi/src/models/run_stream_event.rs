// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RunStreamEvent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStreamEventObject,
    RunStreamEventObject10,
    RunStreamEventObject2,
    RunStreamEventObject3,
    RunStreamEventObject4,
    RunStreamEventObject5,
    RunStreamEventObject6,
    RunStreamEventObject7,
    RunStreamEventObject8,
    RunStreamEventObject9,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunStreamEvent {
    Object(RunStreamEventObject),
    Object2(RunStreamEventObject2),
    Object3(RunStreamEventObject3),
    Object4(RunStreamEventObject4),
    Object5(RunStreamEventObject5),
    Object6(RunStreamEventObject6),
    Object7(RunStreamEventObject7),
    Object8(RunStreamEventObject8),
    Object9(RunStreamEventObject9),
    Object10(RunStreamEventObject10),
}
