// Generated from OpenAPI spec. Do not edit by hand.
//! `Content` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    InputContent,
    OutputContent,
};

/// Multi-modal input and output contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    InputContent(InputContent),
    OutputContent(OutputContent),
}
