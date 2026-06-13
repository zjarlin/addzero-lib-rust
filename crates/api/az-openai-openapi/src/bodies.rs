//! Generic request and response body aliases used by generated OpenAI REST traits.

/// Generic JSON request body.
pub type OpenAiRequestBody = serde_json::Value;
/// Generic JSON response body.
pub type OpenAiResponseBody = serde_json::Value;
/// Generic object-like query value for structured query filters.
pub type OpenAiQueryObject = serde_json::Value;
/// Binary response body for content download endpoints.
pub type OpenAiBinaryBody = Vec<u8>;
/// Text body used by non-JSON endpoints such as SDP.
pub type OpenAiTextBody = String;
