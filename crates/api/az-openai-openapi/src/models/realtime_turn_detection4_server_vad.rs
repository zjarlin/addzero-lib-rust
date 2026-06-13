// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTurnDetection4ServerVAD` DTO.

use serde::{Deserialize, Serialize};

/// Server-side voice activity detection (VAD) which flips on when user speech is detected and off after
/// a period of silence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTurnDetection4ServerVAD {
    /// Type of turn detection, `server_vad` to turn on simple Server VAD.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Used only for `server_vad` mode. Activation threshold for VAD (0.0 to 1.0), this defaults to 0.5. A
    /// higher threshold will require louder audio to activate the model, and thus might perform better in
    /// noisy environments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Used only for `server_vad` mode. Amount of audio to include before the VAD detected speech (in
    /// milliseconds). Defaults to 300ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i32>,
    /// Used only for `server_vad` mode. Duration of silence to detect speech stop (in milliseconds).
    /// Defaults to 500ms. With shorter values the model will respond more quickly, but may jump in on short
    /// pauses from the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i32>,
    /// Whether or not to automatically generate a response when a VAD stop event occurs. If
    /// `interrupt_response` is set to `false` this may fail to create a response if the model is already
    /// responding. If both `create_response` and `interrupt_response` are set to `false`, the model will
    /// never respond automatically but VAD events will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: Option<bool>,
    /// Whether or not to automatically interrupt (cancel) any ongoing response with output to the default
    /// conversation (i.e. `conversation` of `auto`) when a VAD start event occurs. If `true` then the
    /// response will be cancelled, otherwise it will continue until complete. If both `create_response` and
    /// `interrupt_response` are set to `false`, the model will never respond automatically but VAD events
    /// will still be emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle_timeout_ms: Option<i32>,
}
