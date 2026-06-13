// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ChatCompletionStreamOptions2` DTO.

use serde::{Deserialize, Serialize};

/// Options for streaming response. Only set this when you set `stream: true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamOptions2 {
    /// If set, an additional chunk will be streamed before the `data: [DONE]` message. The `usage` field on
    /// this chunk shows the token usage statistics for the entire request, and the `choices` field will
    /// always be an empty array. All other chunks will also include a `usage` field, but with a null value.
    /// **NOTE:** If the stream is interrupted, you may not receive the final usage chunk which contains the
    /// total token usage for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    /// When true, stream obfuscation will be enabled. Stream obfuscation adds random characters to an
    /// `obfuscation` field on streaming delta events to normalize payload sizes as a mitigation to certain
    /// side-channel attacks. These obfuscation fields are included by default, but add a small amount of
    /// overhead to the data stream. You can set `include_obfuscation` to false to optimize for bandwidth if
    /// you trust the network links between your application and the OpenAI API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_obfuscation: Option<bool>,
}
