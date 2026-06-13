// Generated from OpenAPI spec. Do not edit by hand.
//! `ListFineTuningJobEventsResponse` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    FineTuningJobEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFineTuningJobEventsResponse {
    pub data: Vec<FineTuningJobEvent>,
    pub object: String,
    pub has_more: bool,
}
