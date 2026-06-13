// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `WorkflowTracingParam` DTO.

use serde::{Deserialize, Serialize};

/// Controls diagnostic tracing during the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTracingParam {
    /// Whether tracing is enabled during the session. Defaults to true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
