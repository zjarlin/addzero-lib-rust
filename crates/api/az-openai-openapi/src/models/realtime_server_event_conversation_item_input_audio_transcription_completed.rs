// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventConversationItemInputAudioTranscriptionCompleted` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    LogProbProperties,
    RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage,
};

/// This event is the output of audio transcription for user audio written to the user audio buffer.
/// Transcription begins when the input audio buffer is committed by the client or server (when VAD is
/// enabled). Transcription runs asynchronously with Response creation, so this event may come before or
/// after the Response events. Realtime API models accept audio natively, and thus input transcription
/// is a separate process run on a separate ASR (Automatic Speech Recognition) model. The transcript may
/// diverge somewhat from the model's interpretation, and should be treated as a rough guide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventConversationItemInputAudioTranscriptionCompleted {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `conversation.item.input_audio_transcription.completed`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The ID of the item containing the audio that is being transcribed.
    pub item_id: String,
    /// The index of the content part containing the audio.
    pub content_index: i32,
    /// The transcribed text.
    pub transcript: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProbProperties>>,
    /// Usage statistics for the transcription, this is billed according to the ASR model's pricing rather
    /// than the realtime model's pricing.
    pub usage: RealtimeServerEventConversationItemInputAudioTranscriptionCompletedUsage,
}
