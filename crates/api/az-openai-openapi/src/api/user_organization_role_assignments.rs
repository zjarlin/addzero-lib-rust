//! User organization role assignments REST endpoint contract.

use async_trait::async_trait;

use crate::bodies::*;

/// User organization role assignments REST endpoints.
#[async_trait]
pub trait OpenAiUserOrganizationRoleAssignmentsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;
    /// Lists the organization roles assigned to a user within the organization.
    ///
    /// REST: `GET /organization/users/{user_id}/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES).
    async fn list_user_role_assignments(
        &self,
        user_id: String,
        limit: Option<i64>,
        after: Option<String>,
        order: Option<String>,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Assigns an organization role to a user within the organization.
    ///
    /// REST: `POST /organization/users/{user_id}/roles`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES).
    async fn assign_user_role(
        &self,
        user_id: String,
        body: OpenAiRequestBody,
    ) -> Result<OpenAiResponseBody, Self::Error>;

    /// Unassigns an organization role from a user within the organization.
    ///
    /// REST: `DELETE /organization/users/{user_id}/roles/{role_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_USERS_BY_USER_ID_BY_ROLES_BY_ROLE_ID).
    async fn unassign_user_role(
        &self,
        user_id: String,
        role_id: String,
    ) -> Result<OpenAiResponseBody, Self::Error>;
}
