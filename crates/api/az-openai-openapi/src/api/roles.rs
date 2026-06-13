//! Roles REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// Roles REST endpoints.
#[async_trait]
pub trait OpenAiRolesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Lists the roles configured for the organization.
    ///
    /// REST: `GET /organization/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ROLES).
    async fn list_roles(
        &self,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates a custom role for the organization.
    ///
    /// REST: `POST /organization/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ROLES).
    async fn create_role(&self, body: OpenAiRequestBody)
    -> Result<OpenAiResponseBody, Self::Error>;

    /// Deletes a custom role from the organization.
    ///
    /// REST: `DELETE /organization/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID).
    async fn delete_role(&self, role_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Updates an existing organization role.
    ///
    /// REST: `POST /organization/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ROLES_BY_ROLE_ID).
    async fn update_role(
        &self,
        role_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Lists the roles configured for a project.
    ///
    /// REST: `GET /projects/{project_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES).
    async fn list_project_roles(
        &self,
        project_id: String,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates a custom role for a project.
    ///
    /// REST: `POST /projects/{project_id}/roles`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES).
    async fn create_project_role(
        &self,
        project_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deletes a custom role from a project.
    ///
    /// REST: `DELETE /projects/{project_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID).
    async fn delete_project_role(
        &self,
        project_id: String,
        role_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Updates an existing project role.
    ///
    /// REST: `POST /projects/{project_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::PROJECTS_BY_PROJECT_ID_BY_ROLES_BY_ROLE_ID).
    async fn update_project_role(
        &self,
        project_id: String,
        role_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
