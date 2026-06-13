// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `OutputMessageContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputTextContent,
    RefusalContent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputMessageContent {
    OutputTextContent(OutputTextContent),
    RefusalContent(RefusalContent),
}
