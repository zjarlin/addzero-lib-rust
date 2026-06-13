// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponsePromptVariablesValue` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputFileContent,
    InputImageContent,
    InputTextContent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsePromptVariablesValue {
    String(String),
    InputTextContent(InputTextContent),
    InputImageContent(InputImageContent),
    InputFileContent(InputFileContent),
}
