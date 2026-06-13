// Generated from OpenAPI spec. Do not edit by hand.
//! `VideoModel` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VideoModel {
    String(String),
    String2(String),
}
