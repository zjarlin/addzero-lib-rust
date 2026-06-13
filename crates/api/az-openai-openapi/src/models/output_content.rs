// Generated from OpenAPI spec. Do not edit by hand.
//! `OutputContent` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    OutputTextContent,
    ReasoningTextContent,
    RefusalContent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputContent {
    OutputTextContent(OutputTextContent),
    RefusalContent(RefusalContent),
    ReasoningTextContent(ReasoningTextContent),
}
