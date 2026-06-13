// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Shared body aliases used by generated OpenAI REST traits.

/// JSON value used only for schema fields that are intentionally open-ended.
pub type OpenAiJsonValue = serde_json::Value;
/// JSON object used only for schema fields that are intentionally open-ended.
pub type OpenAiJsonObject = std::collections::BTreeMap<String, serde_json::Value>;
/// Binary response body for content download endpoints.
pub type OpenAiBinaryBody = Vec<u8>;
/// Text body used by non-JSON endpoints such as SDP and event streams.
pub type OpenAiTextBody = String;
