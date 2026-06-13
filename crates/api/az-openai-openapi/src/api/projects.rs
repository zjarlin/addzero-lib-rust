// Generated from OpenAPI spec. Do not edit by hand.
//! Projects REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    Project,
    ProjectApiKey,
    ProjectApiKeyDeleteResponse,
    ProjectApiKeyListResponse,
    ProjectCreateRequest,
    ProjectListResponse,
    ProjectRateLimit,
    ProjectRateLimitListResponse,
    ProjectRateLimitUpdateRequest,
    ProjectServiceAccount,
    ProjectServiceAccountCreateRequest,
    ProjectServiceAccountCreateResponse,
    ProjectServiceAccountDeleteResponse,
    ProjectServiceAccountListResponse,
    ProjectUpdateRequest,
    ProjectUser,
    ProjectUserCreateRequest,
    ProjectUserDeleteResponse,
    ProjectUserListResponse,
    ProjectUserUpdateRequest,
};

/// Projects REST endpoints.
#[async_trait]
pub trait OpenAiProjectsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns a list of projects.
    ///
    /// REST: `GET /organization/projects`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS).
    async fn list_projects(
        &self,
        limit: Option<i32>,
        after: Option<String>,
        include_archived: Option<bool>,
    ) -> Result<ProjectListResponse, Self::Error>;

    /// Create a new project in the organization. Projects can be created and archived, but cannot be
    /// deleted.
    ///
    /// REST: `POST /organization/projects`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS).
    async fn create_project(&self, body: ProjectCreateRequest) -> Result<Project, Self::Error>;

    /// Retrieves a project.
    ///
    /// REST: `GET /organization/projects/{project_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID).
    async fn retrieve_project(&self, project_id: String) -> Result<Project, Self::Error>;

    /// Modifies a project in the organization.
    ///
    /// REST: `POST /organization/projects/{project_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID).
    async fn modify_project(
        &self,
        project_id: String,
        body: ProjectUpdateRequest,
    ) -> Result<Project, Self::Error>;

    /// Returns a list of API keys in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/api_keys`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS).
    async fn list_project_api_keys(
        &self,
        project_id: String,
        limit: Option<i32>,
        after: Option<String>,
    ) -> Result<ProjectApiKeyListResponse, Self::Error>;

    /// Retrieves an API key in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/api_keys/{api_key_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID).
    async fn retrieve_project_api_key(
        &self,
        project_id: String,
        api_key_id: String,
    ) -> Result<ProjectApiKey, Self::Error>;

    /// Deletes an API key from the project. Returns confirmation of the key deletion, or an error if the
    /// key belonged to a service account.
    ///
    /// REST: `DELETE /organization/projects/{project_id}/api_keys/{api_key_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_API_KEYS_BY_API_KEY_ID).
    async fn delete_project_api_key(
        &self,
        project_id: String,
        api_key_id: String,
    ) -> Result<ProjectApiKeyDeleteResponse, Self::Error>;

    /// Archives a project in the organization. Archived projects cannot be used or updated.
    ///
    /// REST: `POST /organization/projects/{project_id}/archive`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_ARCHIVE`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_ARCHIVE).
    async fn archive_project(&self, project_id: String) -> Result<Project, Self::Error>;

    /// Returns the rate limits per model for a project.
    ///
    /// REST: `GET /organization/projects/{project_id}/rate_limits`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS).
    async fn list_project_rate_limits(
        &self,
        project_id: String,
        limit: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<ProjectRateLimitListResponse, Self::Error>;

    /// Updates a project rate limit.
    ///
    /// REST: `POST /organization/projects/{project_id}/rate_limits/{rate_limit_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS_BY_RATE_LIMIT_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_RATE_LIMITS_BY_RATE_LIMIT_ID).
    async fn update_project_rate_limits(
        &self,
        project_id: String,
        rate_limit_id: String,
        body: ProjectRateLimitUpdateRequest,
    ) -> Result<ProjectRateLimit, Self::Error>;

    /// Returns a list of service accounts in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/service_accounts`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS).
    async fn list_project_service_accounts(
        &self,
        project_id: String,
        limit: Option<i32>,
        after: Option<String>,
    ) -> Result<ProjectServiceAccountListResponse, Self::Error>;

    /// Creates a new service account in the project. This also returns an unredacted API key for the
    /// service account.
    ///
    /// REST: `POST /organization/projects/{project_id}/service_accounts`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS).
    async fn create_project_service_account(
        &self,
        project_id: String,
        body: ProjectServiceAccountCreateRequest,
    ) -> Result<ProjectServiceAccountCreateResponse, Self::Error>;

    /// Retrieves a service account in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/service_accounts/{service_account_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID).
    async fn retrieve_project_service_account(
        &self,
        project_id: String,
        service_account_id: String,
    ) -> Result<ProjectServiceAccount, Self::Error>;

    /// Deletes a service account from the project. Returns confirmation of service account deletion, or an
    /// error if the project is archived (archived projects have no service accounts).
    ///
    /// REST: `DELETE /organization/projects/{project_id}/service_accounts/{service_account_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_SERVICE_ACCOUNTS_BY_SERVICE_ACCOUNT_ID).
    async fn delete_project_service_account(
        &self,
        project_id: String,
        service_account_id: String,
    ) -> Result<ProjectServiceAccountDeleteResponse, Self::Error>;

    /// Returns a list of users in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/users`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS).
    async fn list_project_users(
        &self,
        project_id: String,
        limit: Option<i32>,
        after: Option<String>,
    ) -> Result<ProjectUserListResponse, Self::Error>;

    /// Adds a user to the project. Users must already be members of the organization to be added to a
    /// project.
    ///
    /// REST: `POST /organization/projects/{project_id}/users`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS).
    async fn create_project_user(
        &self,
        project_id: String,
        body: ProjectUserCreateRequest,
    ) -> Result<ProjectUser, Self::Error>;

    /// Retrieves a user in the project.
    ///
    /// REST: `GET /organization/projects/{project_id}/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID).
    async fn retrieve_project_user(
        &self,
        project_id: String,
        user_id: String,
    ) -> Result<ProjectUser, Self::Error>;

    /// Modifies a user's role in the project.
    ///
    /// REST: `POST /organization/projects/{project_id}/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID).
    async fn modify_project_user(
        &self,
        project_id: String,
        user_id: String,
        body: ProjectUserUpdateRequest,
    ) -> Result<ProjectUser, Self::Error>;

    /// Deletes a user from the project. Returns confirmation of project user deletion, or an error if the
    /// project is archived (archived projects have no users).
    ///
    /// REST: `DELETE /organization/projects/{project_id}/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_PROJECTS_BY_PROJECT_ID_BY_USERS_BY_USER_ID).
    async fn delete_project_user(
        &self,
        project_id: String,
        user_id: String,
    ) -> Result<ProjectUserDeleteResponse, Self::Error>;
}
