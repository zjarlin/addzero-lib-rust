// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeTurnDetection4SemanticVAD` DTO.

use serde::{Deserialize, Serialize};

/// Server-side semantic turn detection which uses a model to determine when the user has finished
/// speaking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTurnDetection4SemanticVAD {
    /// Type of turn detection, `semantic_vad` to turn on Semantic VAD.
    #[serde(rename = "type")]
    pub type_value: String,
    /// Used only for `semantic_vad` mode. The eagerness of the model to respond. `low` will wait longer for
    /// the user to continue speaking, `high` will respond more quickly. `auto` is the default and is
    /// equivalent to `medium`. `low`, `medium`, and `high` have max timeouts of 8s, 4s, and 2s
    /// respectively.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eagerness: Option<String>,
    /// Whether or not to automatically generate a response when a VAD stop event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_response: Option<bool>,
    /// Whether or not to automatically interrupt any ongoing response with output to the default
    /// conversation (i.e. `conversation` of `auto`) when a VAD start event occurs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupt_response: Option<bool>,
}
