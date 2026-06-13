// Generated from OpenAPI spec. Do not edit by hand.
//! `RunStreamEventObject10` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RunObject,
};

/// Occurs when a [run](/docs/api-reference/runs/object) expires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStreamEventObject10 {
    pub event: String,
    pub data: RunObject,
}
