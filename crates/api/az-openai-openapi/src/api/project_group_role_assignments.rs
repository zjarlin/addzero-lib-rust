//! Project group role assignments REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Project group role assignments REST endpoints.
#[async_trait]
pub trait OpenAiProjectGroupRoleAssignmentsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Lists the project roles assigned to a group within a project.
    ///
    /// REST: `GET /projects/{project_id}/groups/{group_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES).
    async fn list_project_group_role_assignments(
        &self,
        project_id: String,
        group_id: String,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Assigns a project role to a group within a project.
    ///
    /// REST: `POST /projects/{project_id}/groups/{group_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES).
    async fn assign_project_group_role(
        &self,
        project_id: String,
        group_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Unassigns a project role from a group within a project.
    ///
    /// REST: `DELETE /projects/{project_id}/groups/{group_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID_BY_ROLES_BY_ROLE_ID).
    async fn unassign_project_group_role(
        &self,
        project_id: String,
        group_id: String,
        role_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
