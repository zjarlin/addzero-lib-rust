// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLogOrganizationUpdatedChangesRequested` DTO.

use serde::{Deserialize, Serialize};

/// The payload used to update the organization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogOrganizationUpdatedChangesRequested {
    /// The organization title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The organization description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The organization name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Visibility of the threads page which shows messages created with the Assistants API and Playground.
    /// One of `ANY_ROLE`, `OWNERS`, or `NONE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads_ui_visibility: Option<String>,
    /// Visibility of the usage dashboard which shows activity and costs for your organization. One of
    /// `ANY_ROLE` or `OWNERS`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_dashboard_visibility: Option<String>,
    /// How your organization logs data from supported API calls. One of `disabled`, `enabled_per_call`,
    /// `enabled_for_all_projects`, or `enabled_for_selected_projects`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_call_logging: Option<String>,
    /// The list of project ids if api_call_logging is set to `enabled_for_selected_projects`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_call_logging_project_ids: Option<String>,
}
