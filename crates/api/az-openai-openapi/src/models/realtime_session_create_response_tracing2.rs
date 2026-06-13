// Generated from OpenAPI spec. Do not edit by hand.
//! `RealtimeSessionCreateResponseTracing2` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    RealtimeSessionCreateResponseTracing2TracingConfiguration,
};

/// Configuration options for tracing. Set to null to disable tracing. Once tracing is enabled for a
/// session, the configuration cannot be modified. `auto` will create a trace for the session with
/// default values for the workflow name, group id, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RealtimeSessionCreateResponseTracing2 {
    Auto(String),
    TracingConfiguration(RealtimeSessionCreateResponseTracing2TracingConfiguration),
}
