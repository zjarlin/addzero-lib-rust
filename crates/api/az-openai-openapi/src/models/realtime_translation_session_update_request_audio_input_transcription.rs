// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeTranslationSessionUpdateRequestAudioInputTranscription` DTO.

use serde::{Deserialize, Serialize};

/// Optional source-language transcription. When configured, the server emits
/// `session.input_transcript.delta` events. Translation itself still runs from the input audio stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTranslationSessionUpdateRequestAudioInputTranscription {
    /// The transcription model to use for source transcript deltas.
    pub model: String,
}
