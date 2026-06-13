// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeAudioFormatsPCMUAudioFormat` DTO.

use serde::{Deserialize, Serialize};

/// The G.711 μ-law format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAudioFormatsPCMUAudioFormat {
    /// The audio format. Always `audio/pcmu`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
}
