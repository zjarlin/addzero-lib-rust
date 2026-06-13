// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTranslationSessionCreateRequest` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTranslationSessionCreateRequestAudio,
};

/// Realtime translation session configuration. Translation sessions stream source audio in and
/// translated audio plus transcript deltas out continuously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionCreateRequest {
    /// The Realtime translation model used for this session.
    pub model: String,
    /// Configuration for translation input and output audio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<RealtimeTranslationSessionCreateRequestAudio>,
}
