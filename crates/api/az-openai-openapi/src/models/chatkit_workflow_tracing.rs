// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatkitWorkflowTracing` DTO.

use serde::{Deserialize, Serialize};

/// Controls diagnostic tracing during the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatkitWorkflowTracing {
    /// Indicates whether tracing is enabled.
    pub enabled: bool,
}
