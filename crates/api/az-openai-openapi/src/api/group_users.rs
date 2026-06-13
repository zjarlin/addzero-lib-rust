// Generated from OpenAPI spec. Do not edit by hand.
//! GroupUsers REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateGroupUserBody,
    GroupUserAssignment,
    GroupUserDeletedResource,
    UserListResource,
};

/// GroupUsers REST endpoints.
#[async_trait]
pub trait OpenAiGroupUsersApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists the users assigned to a group.
    ///
    /// REST: `GET /organization/groups/{group_id}/users`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS).
    async fn list_group_users(
        &self,
        group_id: String,
        limit: Option<i32>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<UserListResource, Self::Error>;

    /// Adds a user to a group.
    ///
    /// REST: `POST /organization/groups/{group_id}/users`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS).
    async fn add_group_user(
        &self,
        group_id: String,
        body: CreateGroupUserBody,
    ) -> Result<GroupUserAssignment, Self::Error>;

    /// Removes a user from a group.
    ///
    /// REST: `DELETE /organization/groups/{group_id}/users/{user_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS_BY_USER_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID_BY_USERS_BY_USER_ID).
    async fn remove_group_user(
        &self,
        group_id: String,
        user_id: String,
    ) -> Result<GroupUserDeletedResource, Self::Error>;
}
