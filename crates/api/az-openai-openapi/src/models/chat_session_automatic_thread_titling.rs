// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatSessionAutomaticThreadTitling` DTO.

use serde::{Deserialize, Serialize};

/// Automatic thread title preferences for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionAutomaticThreadTitling {
    /// Whether automatic thread titling is enabled.
    pub enabled: bool,
}
