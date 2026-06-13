// Generated from OpenAPI spec. Do not edit by hand.
//! Invites REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    Invite,
    InviteDeleteResponse,
    InviteListResponse,
    InviteRequest,
};

/// Invites REST endpoints.
#[async_trait]
pub trait OpenAiInvitesApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns a list of invites in the organization.
    ///
    /// REST: `GET /organization/invites`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_INVITES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_INVITES).
    async fn list_invites(
        &self,
        limit: Option<i32>,
        after: Option<String>,
    ) -> Result<InviteListResponse, Self::Error>;

    /// Create an invite for a user to the organization. The invite must be accepted by the user before they
    /// have access to the organization.
    ///
    /// REST: `POST /organization/invites`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_INVITES`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_INVITES).
    async fn invite_user(&self, body: InviteRequest) -> Result<Invite, Self::Error>;

    /// Retrieves an invite.
    ///
    /// REST: `GET /organization/invites/{invite_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID).
    async fn retrieve_invite(&self, invite_id: String) -> Result<Invite, Self::Error>;

    /// Delete an invite. If the invite has already been accepted, it cannot be deleted.
    ///
    /// REST: `DELETE /organization/invites/{invite_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_INVITES_BY_INVITE_ID).
    async fn delete_invite(&self, invite_id: String) -> Result<InviteDeleteResponse, Self::Error>;
}
