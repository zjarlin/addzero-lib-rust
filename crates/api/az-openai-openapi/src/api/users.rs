// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! Users REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    User,
    UserDeleteResponse,
    UserListResponse,
    UserRoleUpdateRequest,
};

/// Users REST endpoints.
#[async_trait]
pub trait OpenAiUsersApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists all of the users in the organization.
    ///
    /// REST: `GET /organization/users`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS).
    async fn list_users(
        &self,
        limit: Option<i32>,
        after: Option<String>,
        emails: Option<Vec<String>>,
    ) -> Result<UserListResponse, Self::Error>;

    /// Retrieves a user by their identifier.
    ///
    /// REST: `GET /organization/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID).
    async fn retrieve_user(&self, user_id: String) -> Result<User, Self::Error>;

    /// Modifies a user's role in the organization.
    ///
    /// REST: `POST /organization/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID).
    async fn modify_user(
        &self,
        user_id: String,
        body: UserRoleUpdateRequest,
    ) -> Result<User, Self::Error>;

    /// Deletes a user from the organization.
    ///
    /// REST: `DELETE /organization/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID).
    async fn delete_user(&self, user_id: String) -> Result<UserDeleteResponse, Self::Error>;
}
