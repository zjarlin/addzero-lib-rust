// Generated from OpenAPI spec. Do not edit by hand.
//! `ChatkitWorkflow` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ChatkitWorkflowStateVariablesValue,
    ChatkitWorkflowTracing,
};

/// Workflow metadata and state returned for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatkitWorkflow {
    /// Identifier of the workflow backing the session.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_variables: Option<std::collections::BTreeMap<String, ChatkitWorkflowStateVariablesValue>>,
    /// Tracing settings applied to the workflow.
    pub tracing: ChatkitWorkflowTracing,
}
