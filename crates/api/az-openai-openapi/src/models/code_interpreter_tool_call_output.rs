// Generated from OpenAPI spec. Do not edit by hand.
//! `CodeInterpreterToolCallOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CodeInterpreterOutputImage,
    CodeInterpreterOutputLogs,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterToolCallOutput {
    CodeInterpreterOutputLogs(CodeInterpreterOutputLogs),
    CodeInterpreterOutputImage(CodeInterpreterOutputImage),
}
