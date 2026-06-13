// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeMCPToolCallError` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeMCPHTTPError,
    RealtimeMCPProtocolError,
    RealtimeMCPToolExecutionError,
};

/// The error from the tool call, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeMCPToolCallError {
    RealtimeMCPProtocolError(RealtimeMCPProtocolError),
    RealtimeMCPToolExecutionError(RealtimeMCPToolExecutionError),
    RealtimeMCPHTTPError(RealtimeMCPHTTPError),
}
