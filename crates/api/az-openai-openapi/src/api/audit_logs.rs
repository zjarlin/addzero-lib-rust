// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! AuditLogs REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    AuditLogEventType,
    EffectiveAtParameter,
    ListAuditLogsResponse,
};

/// AuditLogs REST endpoints.
#[async_trait]
pub trait OpenAiAuditLogsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List user actions and configuration changes within this organization.
    ///
    /// REST: `GET /organization/audit_logs`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_AUDIT_LOGS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_AUDIT_LOGS).
    async fn list_audit_logs(
        &self,
        effective_at: Option<EffectiveAtParameter>,
        project_ids: Option<Vec<String>>,
        event_types: Option<Vec<AuditLogEventType>>,
        actor_ids: Option<Vec<String>>,
        actor_emails: Option<Vec<String>>,
        resource_ids: Option<Vec<String>>,
        limit: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ListAuditLogsResponse, Self::Error>;
}
