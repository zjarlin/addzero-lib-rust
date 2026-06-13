// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessageContentTextAnnotationsFileCitationObjectFileCitation` DTO.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentTextAnnotationsFileCitationObjectFileCitation {
    /// The ID of the specific File the citation is from.
    pub file_id: String,
}
