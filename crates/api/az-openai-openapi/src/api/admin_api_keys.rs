// Generated from OpenAPI spec. Do not edit by hand.
//! AdminApiKeys REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    AdminApiKey,
    AdminApiKeyCreateResponse,
    AdminApiKeysCreateRequest,
    AdminApiKeysDeleteResponse,
    ApiKeyList,
};

/// AdminApiKeys REST endpoints.
#[async_trait]
pub trait OpenAiAdminApiKeysApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List organization API keys
    ///
    /// REST: `GET /organization/admin_api_keys`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS).
    async fn admin_api_keys_list(
        &self,
        after: Option<String>,
        order: Option<String>,
        limit: Option<i32>,
    ) -> Result<ApiKeyList, Self::Error>;

    /// Create an organization admin API key
    ///
    /// REST: `POST /organization/admin_api_keys`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS).
    async fn admin_api_keys_create(
        &self,
        body: AdminApiKeysCreateRequest,
    ) -> Result<AdminApiKeyCreateResponse, Self::Error>;

    /// Retrieve a single organization API key
    ///
    /// REST: `GET /organization/admin_api_keys/{key_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID).
    async fn admin_api_keys_get(&self, key_id: String) -> Result<AdminApiKey, Self::Error>;

    /// Delete an organization admin API key
    ///
    /// REST: `DELETE /organization/admin_api_keys/{key_id}`.
    /// Path constant: [`OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID`](crate::paths::OpenAiApiPath::ORGANIZATION_BY_ADMIN_API_KEYS_BY_KEY_ID).
    async fn admin_api_keys_delete(
        &self,
        key_id: String,
    ) -> Result<AdminApiKeysDeleteResponse, Self::Error>;
}
