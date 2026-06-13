// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WaitParam` DTO.

use serde::{Deserialize, Serialize};

/// A wait action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitParam {
    /// Specifies the event type. For a wait action, this property is always set to `wait`.
    #[serde(rename = "type")]
    pub type_value: String,
}
