// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `InputContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputFileContent,
    InputImageContent,
    InputTextContent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputContent {
    InputTextContent(InputTextContent),
    InputImageContent(InputImageContent),
    InputFileContent(InputFileContent),
}
