// Generated from OpenAPI spec. Do not edit by hand.
//! ProjectUserRoleAssignments REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    DeletedRoleAssignmentResource,
    PublicAssignOrganizationGroupRoleBody,
    RoleListResource,
    UserRoleAssignment,
};

/// ProjectUserRoleAssignments REST endpoints.
#[async_trait]
pub trait OpenAiProjectUserRoleAssignmentsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists the project roles assigned to a user within a project.
    ///
    /// REST: `GET /projects/{project_id}/users/{user_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES).
    async fn list_project_user_role_assignments(
        &self,
        project_id: String,
        user_id: String,
        limit: Option<i32>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<RoleListResource, Self::Error>;

    /// Assigns a project role to a user within a project.
    ///
    /// REST: `POST /projects/{project_id}/users/{user_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES).
    async fn assign_project_user_role(
        &self,
        project_id: String,
        user_id: String,
        body: PublicAssignOrganizationGroupRoleBody,
    ) -> Result<UserRoleAssignment, Self::Error>;

    /// Unassigns a project role from a user within a project.
    ///
    /// REST: `DELETE /projects/{project_id}/users/{user_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID).
    async fn unassign_project_user_role(
        &self,
        project_id: String,
        user_id: String,
        role_id: String,
    ) -> Result<DeletedRoleAssignmentResource, Self::Error>;
}
