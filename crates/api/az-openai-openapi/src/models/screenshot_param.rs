// Generated from OpenAPI spec. Do not edit by hand.
//! `ScreenshotParam` DTO.

use serde::{Deserialize, Serialize};

/// A screenshot action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotParam {
    /// Specifies the event type. For a screenshot action, this property is always set to `screenshot`.
    #[serde(rename = "type")]
    pub type_value: String,
}
