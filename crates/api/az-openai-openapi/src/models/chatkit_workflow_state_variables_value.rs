// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatkitWorkflowStateVariablesValue` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatkitWorkflowStateVariablesValue {
    String(String),
    Integer(i32),
    Boolean(bool),
    Number(f64),
}
