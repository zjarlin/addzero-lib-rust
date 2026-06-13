// Generated from OpenAPI spec. Do not edit by hand.
//! GroupOrganizationRoleAssignments REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    DeletedRoleAssignmentResource,
    GroupRoleAssignment,
    PublicAssignOrganizationGroupRoleBody,
    RoleListResource,
};

/// GroupOrganizationRoleAssignments REST endpoints.
#[async_trait]
pub trait OpenAiGroupOrganizationRoleAssignmentsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists the organization roles assigned to a group within the organization.
    ///
    /// REST: `GET /organization/groups/{group_id}/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES).
    async fn list_group_role_assignments(
        &self,
        group_id: String,
        limit: Option<i32>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<RoleListResource, Self::Error>;

    /// Assigns an organization role to a group within the organization.
    ///
    /// REST: `POST /organization/groups/{group_id}/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES).
    async fn assign_group_role(
        &self,
        group_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<GroupRoleAssignment, Self::Error>;

    /// Unassigns an organization role from a group within the organization.
    ///
    /// REST: `DELETE /organization/groups/{group_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID).
    async fn unassign_group_role(
        &self,
        group_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
