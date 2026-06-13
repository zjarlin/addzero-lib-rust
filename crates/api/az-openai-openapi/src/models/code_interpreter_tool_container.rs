// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CodeInterpreterToolContainer` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    AutoCodeInterpreterToolParam,
};

/// The code interpreter container. Can be a container ID or an object that specifies uploaded file IDs
/// to make available to your code, along with an optional `memory_limit` setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodeInterpreterToolContainer {
    String(String),
    AutoCodeInterpreterToolParam(AutoCodeInterpreterToolParam),
}
