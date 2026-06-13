// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeAudioFormatsPCMAAudioFormat` DTO.

use serde::{Deserialize, Serialize};

/// The G.711 A-law format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeAudioFormatsPCMAAudioFormat {
    /// The audio format. Always `audio/pcma`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub type_value: Option<String>,
}
