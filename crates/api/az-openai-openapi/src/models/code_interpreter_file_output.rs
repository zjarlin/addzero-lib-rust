// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `CodeInterpreterFileOutput` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    CodeInterpreterFileOutputFile,
};

/// The output of a code interpreter tool call that is a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInterpreterFileOutput {
    /// The type of the code interpreter file output. Always `files`.
    #[serde(rename = "type")]
    pub type_value: String,
    pub files: Vec<CodeInterpreterFileOutputFile>,
}
