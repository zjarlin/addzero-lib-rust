// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `AuditLog` DTO.

use serde::{Deserialize, Serialize};

use crate::bodies::{
    OpenAiJsonObject,
};

use crate::models::{
    AuditLogActor,
    AuditLogApiKeyCreated,
    AuditLogApiKeyDeleted,
    AuditLogApiKeyUpdated,
    AuditLogCertificateCreated,
    AuditLogCertificateDeleted,
    AuditLogCertificateUpdated,
    AuditLogCertificatesActivated,
    AuditLogCertificatesDeactivated,
    AuditLogCheckpointPermissionCreated,
    AuditLogCheckpointPermissionDeleted,
    AuditLogEventType,
    AuditLogExternalKeyRegistered,
    AuditLogExternalKeyRemoved,
    AuditLogGroupCreated,
    AuditLogGroupDeleted,
    AuditLogGroupUpdated,
    AuditLogInviteAccepted,
    AuditLogInviteDeleted,
    AuditLogInviteSent,
    AuditLogIpAllowlistConfigActivated,
    AuditLogIpAllowlistConfigDeactivated,
    AuditLogIpAllowlistCreated,
    AuditLogIpAllowlistDeleted,
    AuditLogIpAllowlistUpdated,
    AuditLogLoginFailed,
    AuditLogLogoutFailed,
    AuditLogOrganizationUpdated,
    AuditLogProject,
    AuditLogProjectArchived,
    AuditLogProjectCreated,
    AuditLogProjectDeleted,
    AuditLogProjectUpdated,
    AuditLogRateLimitDeleted,
    AuditLogRateLimitUpdated,
    AuditLogRoleAssignmentCreated,
    AuditLogRoleAssignmentDeleted,
    AuditLogRoleCreated,
    AuditLogRoleDeleted,
    AuditLogRoleUpdated,
    AuditLogScimDisabled,
    AuditLogScimEnabled,
    AuditLogServiceAccountCreated,
    AuditLogServiceAccountDeleted,
    AuditLogServiceAccountUpdated,
    AuditLogUserAdded,
    AuditLogUserDeleted,
    AuditLogUserUpdated,
};

/// A log of a user action or configuration change within this organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// The ID of this log.
    pub id: String,
    #[serde(rename = "type")]
    pub type_value: AuditLogEventType,
    /// The Unix timestamp (in seconds) of the event.
    pub effective_at: i64,
    /// The project that the action was scoped to. Absent for actions not scoped to projects. Note that any
    /// admin actions taken via Admin API keys are associated with the default project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<AuditLogProject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<AuditLogActor>,
    /// The details for events with this `type`.
    #[serde(rename = "api_key.created", default, skip_serializing_if = "Option::is_none")]
    pub api_key_created: Option<AuditLogApiKeyCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "api_key.updated", default, skip_serializing_if = "Option::is_none")]
    pub api_key_updated: Option<AuditLogApiKeyUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "api_key.deleted", default, skip_serializing_if = "Option::is_none")]
    pub api_key_deleted: Option<AuditLogApiKeyDeleted>,
    /// The project and fine-tuned model checkpoint that the checkpoint permission was created for.
    #[serde(rename = "checkpoint.permission.created", default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_permission_created: Option<AuditLogCheckpointPermissionCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "checkpoint.permission.deleted", default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_permission_deleted: Option<AuditLogCheckpointPermissionDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "external_key.registered", default, skip_serializing_if = "Option::is_none")]
    pub external_key_registered: Option<AuditLogExternalKeyRegistered>,
    /// The details for events with this `type`.
    #[serde(rename = "external_key.removed", default, skip_serializing_if = "Option::is_none")]
    pub external_key_removed: Option<AuditLogExternalKeyRemoved>,
    /// The details for events with this `type`.
    #[serde(rename = "group.created", default, skip_serializing_if = "Option::is_none")]
    pub group_created: Option<AuditLogGroupCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "group.updated", default, skip_serializing_if = "Option::is_none")]
    pub group_updated: Option<AuditLogGroupUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "group.deleted", default, skip_serializing_if = "Option::is_none")]
    pub group_deleted: Option<AuditLogGroupDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "scim.enabled", default, skip_serializing_if = "Option::is_none")]
    pub scim_enabled: Option<AuditLogScimEnabled>,
    /// The details for events with this `type`.
    #[serde(rename = "scim.disabled", default, skip_serializing_if = "Option::is_none")]
    pub scim_disabled: Option<AuditLogScimDisabled>,
    /// The details for events with this `type`.
    #[serde(rename = "invite.sent", default, skip_serializing_if = "Option::is_none")]
    pub invite_sent: Option<AuditLogInviteSent>,
    /// The details for events with this `type`.
    #[serde(rename = "invite.accepted", default, skip_serializing_if = "Option::is_none")]
    pub invite_accepted: Option<AuditLogInviteAccepted>,
    /// The details for events with this `type`.
    #[serde(rename = "invite.deleted", default, skip_serializing_if = "Option::is_none")]
    pub invite_deleted: Option<AuditLogInviteDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "ip_allowlist.created", default, skip_serializing_if = "Option::is_none")]
    pub ip_allowlist_created: Option<AuditLogIpAllowlistCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "ip_allowlist.updated", default, skip_serializing_if = "Option::is_none")]
    pub ip_allowlist_updated: Option<AuditLogIpAllowlistUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "ip_allowlist.deleted", default, skip_serializing_if = "Option::is_none")]
    pub ip_allowlist_deleted: Option<AuditLogIpAllowlistDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "ip_allowlist.config.activated", default, skip_serializing_if = "Option::is_none")]
    pub ip_allowlist_config_activated: Option<AuditLogIpAllowlistConfigActivated>,
    /// The details for events with this `type`.
    #[serde(rename = "ip_allowlist.config.deactivated", default, skip_serializing_if = "Option::is_none")]
    pub ip_allowlist_config_deactivated: Option<AuditLogIpAllowlistConfigDeactivated>,
    /// This event has no additional fields beyond the standard audit log attributes.
    #[serde(rename = "login.succeeded", default, skip_serializing_if = "Option::is_none")]
    pub login_succeeded: Option<OpenAiJsonObject>,
    /// The details for events with this `type`.
    #[serde(rename = "login.failed", default, skip_serializing_if = "Option::is_none")]
    pub login_failed: Option<AuditLogLoginFailed>,
    /// This event has no additional fields beyond the standard audit log attributes.
    #[serde(rename = "logout.succeeded", default, skip_serializing_if = "Option::is_none")]
    pub logout_succeeded: Option<OpenAiJsonObject>,
    /// The details for events with this `type`.
    #[serde(rename = "logout.failed", default, skip_serializing_if = "Option::is_none")]
    pub logout_failed: Option<AuditLogLogoutFailed>,
    /// The details for events with this `type`.
    #[serde(rename = "organization.updated", default, skip_serializing_if = "Option::is_none")]
    pub organization_updated: Option<AuditLogOrganizationUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "project.created", default, skip_serializing_if = "Option::is_none")]
    pub project_created: Option<AuditLogProjectCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "project.updated", default, skip_serializing_if = "Option::is_none")]
    pub project_updated: Option<AuditLogProjectUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "project.archived", default, skip_serializing_if = "Option::is_none")]
    pub project_archived: Option<AuditLogProjectArchived>,
    /// The details for events with this `type`.
    #[serde(rename = "project.deleted", default, skip_serializing_if = "Option::is_none")]
    pub project_deleted: Option<AuditLogProjectDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "rate_limit.updated", default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_updated: Option<AuditLogRateLimitUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "rate_limit.deleted", default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_deleted: Option<AuditLogRateLimitDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "role.created", default, skip_serializing_if = "Option::is_none")]
    pub role_created: Option<AuditLogRoleCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "role.updated", default, skip_serializing_if = "Option::is_none")]
    pub role_updated: Option<AuditLogRoleUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "role.deleted", default, skip_serializing_if = "Option::is_none")]
    pub role_deleted: Option<AuditLogRoleDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "role.assignment.created", default, skip_serializing_if = "Option::is_none")]
    pub role_assignment_created: Option<AuditLogRoleAssignmentCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "role.assignment.deleted", default, skip_serializing_if = "Option::is_none")]
    pub role_assignment_deleted: Option<AuditLogRoleAssignmentDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "service_account.created", default, skip_serializing_if = "Option::is_none")]
    pub service_account_created: Option<AuditLogServiceAccountCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "service_account.updated", default, skip_serializing_if = "Option::is_none")]
    pub service_account_updated: Option<AuditLogServiceAccountUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "service_account.deleted", default, skip_serializing_if = "Option::is_none")]
    pub service_account_deleted: Option<AuditLogServiceAccountDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "user.added", default, skip_serializing_if = "Option::is_none")]
    pub user_added: Option<AuditLogUserAdded>,
    /// The details for events with this `type`.
    #[serde(rename = "user.updated", default, skip_serializing_if = "Option::is_none")]
    pub user_updated: Option<AuditLogUserUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "user.deleted", default, skip_serializing_if = "Option::is_none")]
    pub user_deleted: Option<AuditLogUserDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "certificate.created", default, skip_serializing_if = "Option::is_none")]
    pub certificate_created: Option<AuditLogCertificateCreated>,
    /// The details for events with this `type`.
    #[serde(rename = "certificate.updated", default, skip_serializing_if = "Option::is_none")]
    pub certificate_updated: Option<AuditLogCertificateUpdated>,
    /// The details for events with this `type`.
    #[serde(rename = "certificate.deleted", default, skip_serializing_if = "Option::is_none")]
    pub certificate_deleted: Option<AuditLogCertificateDeleted>,
    /// The details for events with this `type`.
    #[serde(rename = "certificates.activated", default, skip_serializing_if = "Option::is_none")]
    pub certificates_activated: Option<AuditLogCertificatesActivated>,
    /// The details for events with this `type`.
    #[serde(rename = "certificates.deactivated", default, skip_serializing_if = "Option::is_none")]
    pub certificates_deactivated: Option<AuditLogCertificatesDeactivated>,
}
