// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventInputAudioBufferTimeoutTriggered` DTO.

use serde::{Deserialize, Serialize};

/// Returned when the Server VAD timeout is triggered for the input audio buffer. This is configured
/// with `idle_timeout_ms` in the `turn_detection` settings of the session, and it indicates that there
/// hasn't been any speech detected for the configured duration. The `audio_start_ms` and `audio_end_ms`
/// fields indicate the segment of audio after the last model response up to the triggering time, as an
/// offset from the beginning of audio written to the input audio buffer. This means it demarcates the
/// segment of audio that was silent and the difference between the start and end values will roughly
/// match the configured timeout. The empty audio will be committed to the conversation as an
/// `input_audio` item (there will be a `input_audio_buffer.committed` event) and a model response will
/// be generated. There may be speech that didn't trigger VAD but is still detected by the model, so the
/// model may respond with something relevant to the conversation or a prompt to continue speaking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventInputAudioBufferTimeoutTriggered {
    /// The unique ID of the server event.
    pub event_id: String,
    /// The event type, must be `input_audio_buffer.timeout_triggered`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Millisecond offset of audio written to the input audio buffer that was after the playback time of
    /// the last model response.
    pub audio_start_ms: i32,
    /// Millisecond offset of audio written to the input audio buffer at the time the timeout was triggered.
    pub audio_end_ms: i32,
    /// The ID of the item associated with this segment.
    pub item_id: String,
}
