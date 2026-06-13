// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTurnDetection4` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeTurnDetection4SemanticVAD,
    RealtimeTurnDetection4ServerVAD,
};

/// Configuration for turn detection, ether Server VAD or Semantic VAD. This can be set to `null` to
/// turn off, in which case the client must manually trigger model response. Server VAD means that the
/// model will detect the start and end of speech based on audio volume and respond at the end of user
/// speech. Semantic VAD is more advanced and uses a turn detection model (in conjunction with VAD) to
/// semantically estimate whether the user has finished speaking, then dynamically sets a timeout based
/// on this probability. For example, if user audio trails off with "uhhm", the model will score a low
/// probability of turn end and wait longer for the user to continue speaking. This can be useful for
/// more natural conversations, but may have a higher latency. For `gpt-realtime-whisper` transcription
/// sessions, turn detection must be set to `null`; VAD is not supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeTurnDetection4 {
    ServerVAD(RealtimeTurnDetection4ServerVAD),
    SemanticVAD(RealtimeTurnDetection4SemanticVAD),
}
