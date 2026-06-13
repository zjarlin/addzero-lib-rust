// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `RealtimeServerEventInputAudioBufferDtmfEventReceived` DTO.

use serde::{Deserialize, Serialize};

/// **SIP Only:** Returned when an DTMF event is received. A DTMF event is a message that represents a
/// telephone keypad press (0–9, *, #, A–D). The `event` property is the keypad that the user press. The
/// `received_at` is the UTC Unix Timestamp that the server received the event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeServerEventInputAudioBufferDtmfEventReceived {
    /// The event type, must be `input_audio_buffer.dtmf_event_received`.
    #[serde(rename = "type")]
    pub type_value: String,
    /// The telephone keypad that was pressed by the user.
    pub event: String,
    /// UTC Unix Timestamp when DTMF Event was received by server.
    pub received_at: i32,
}
