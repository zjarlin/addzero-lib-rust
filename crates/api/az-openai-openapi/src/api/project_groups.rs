//! Project groups REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Project groups REST endpoints.
#[async_trait]
pub trait OpenAiProjectGroupsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Lists the groups that have access to a project.
    ///
    /// REST: `GET /organization/projects/{project_id}/groups`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS).
    async fn list_project_groups(
        &self,
        project_id: String,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Grants a group access to a project.
    ///
    /// REST: `POST /organization/projects/{project_id}/groups`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS).
    async fn add_project_group(
        &self,
        project_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Revokes a group's access to a project.
    ///
    /// REST: `DELETE /organization/projects/{project_id}/groups/{group_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_GROUPS_BY_GROUP_ID).
    async fn remove_project_group(
        &self,
        project_id: String,
        group_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
