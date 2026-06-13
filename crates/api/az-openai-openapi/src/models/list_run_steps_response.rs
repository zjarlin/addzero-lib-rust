// Generated from OpenAPI spec. Do not edit by hand.
//! `ListRunStepsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunStepObject,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRunStepsResponse {
    pub object: String,
    pub data: Vec<RunStepObject>,
    pub first_id: String,
    pub last_id: String,
    pub has_more: bool,
}
