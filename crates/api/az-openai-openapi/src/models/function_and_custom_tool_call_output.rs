// Generated from OpenAPI spec. Do not edit by hand.
//! `FunctionAndCustomToolCallOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputFileContent,
    InputImageContent,
    InputTextContent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionAndCustomToolCallOutput {
    InputTextContent(InputTextContent),
    InputImageContent(InputImageContent),
    InputFileContent(InputFileContent),
}
