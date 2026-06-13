// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatSessionChatkitConfiguration` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatSessionAutomaticThreadTitling,
    ChatSessionFileUpload,
    ChatSessionHistory,
};

/// ChatKit configuration for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionChatkitConfiguration {
    /// Automatic thread titling preferences.
    pub automatic_thread_titling: ChatSessionAutomaticThreadTitling,
    /// Upload settings for the session.
    pub file_upload: ChatSessionFileUpload,
    /// History retention configuration.
    pub history: ChatSessionHistory,
}
