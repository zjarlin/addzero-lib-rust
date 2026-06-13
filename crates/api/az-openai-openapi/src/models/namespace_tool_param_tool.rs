// Generated from OpenAPI spec. Do not edit by hand.
//! `NamespaceToolParamTool` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CustomToolParam,
    FunctionToolParam,
};

/// A function or custom tool that belongs to a namespace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NamespaceToolParamTool {
    FunctionToolParam(FunctionToolParam),
    CustomToolParam(CustomToolParam),
}
