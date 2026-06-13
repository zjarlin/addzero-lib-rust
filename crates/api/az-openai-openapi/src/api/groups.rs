// Generated from OpenAPI spec. Do not edit by hand.
//! Groups REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateGroupBody,
    GroupDeletedResource,
    GroupListResource,
    GroupResourceWithSuccess,
    GroupResponse,
    UpdateGroupBody,
};

/// Groups REST endpoints.
#[async_trait]
pub trait OpenAiGroupsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Lists all groups in the organization.
    ///
    /// REST: `GET /organization/groups`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS).
    async fn list_groups(
        &self,
        limit: Option<i32>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<GroupListResource, Self::Error>;

    /// Creates a new group in the organization.
    ///
    /// REST: `POST /organization/groups`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS).
    async fn create_group(&self, body: CreateGroupBody) -> Result<GroupResponse, Self::Error>;

    /// Updates a group's information.
    ///
    /// REST: `POST /organization/groups/{group_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID).
    async fn update_group(
        &self,
        group_id: String,
        body: UpdateGroupBody,
    ) -> Result<GroupResourceWithSuccess, Self::Error>;

    /// Deletes a group from the organization.
    ///
    /// REST: `DELETE /organization/groups/{group_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID).
    async fn delete_group(&self, group_id: String) -> Result<GroupDeletedResource, Self::Error>;
}
