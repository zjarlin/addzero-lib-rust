// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WorkflowParamStateVariablesValue` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkflowParamStateVariablesValue {
    String(String),
    Integer(i32),
    Boolean(bool),
    Number(f64),
}
