// Generated from OpenAPI spec. Do not edit by hand.
//! `ResponseStreamOptions2` DTO.

use serde::{Deserialize, Serialize};

/// Options for streaming responses. Only set this when you set `stream: true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStreamOptions2 {
    /// When true, stream obfuscation will be enabled. Stream obfuscation adds random characters to an
    /// `obfuscation` field on streaming delta events to normalize payload sizes as a mitigation to certain
    /// side-channel attacks. These obfuscation fields are included by default, but add a small amount of
    /// overhead to the data stream. You can set `include_obfuscation` to false to optimize for bandwidth if
    /// you trust the network links between your application and the OpenAI API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}
