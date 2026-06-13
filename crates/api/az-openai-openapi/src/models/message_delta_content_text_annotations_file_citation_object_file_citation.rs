// Generated from OpenAPI spec. Do not edit by hand.
//! `MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaContentTextAnnotationsFileCitationObjectFileCitation {
    /// The ID of the specific File the citation is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// The specific quote in the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}
