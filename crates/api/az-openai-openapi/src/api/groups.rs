//! Groups REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

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
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Creates a new group in the organization.
    ///
    /// REST: `POST /organization/groups`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS).
    async fn create_group(
        &self,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Deletes a group from the organization.
    ///
    /// REST: `DELETE /organization/groups/{group_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID).
    async fn delete_group(&self, group_id: String) -> Result<OpenAiResponseBody, Self::Error>;

    /// Updates a group's information.
    ///
    /// REST: `POST /organization/groups/{group_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_GROUPS_BY_GROUP_ID).
    async fn update_group(
        &self,
        group_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
