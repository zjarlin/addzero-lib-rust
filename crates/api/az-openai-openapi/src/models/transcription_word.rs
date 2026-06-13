// Generated from OpenAPI spec. Do not edit by hand.
//! `TranscriptionWord` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    /// The text content of the word.
    pub word: String,
    /// Start time of the word in seconds.
    pub start: f64,
    /// End time of the word in seconds.
    pub end: f64,
}
