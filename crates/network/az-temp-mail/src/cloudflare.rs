use crate::util::{required_non_blank, trim_non_blank};
use crate::{
    ApiConfig, CloudflareTempMailApi, CreateMailboxRequest, NewAddressRequest, TempMailResult,
};

/// Consumer-supplied context for initializing a Cloudflare temp-mail worker client
/// and its default mailbox creation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareTempMailContext {
    /// Base URL of the deployed worker, for example `https://mail.example.com`.
    pub base_url: String,
    /// Optional deployment-specific header value sent as `x-custom-auth`.
    pub custom_auth: Option<String>,
    /// Optional preferred mailbox local part.
    pub address_name: Option<String>,
    /// Optional preferred mailbox domain.
    pub address_domain: Option<String>,
    /// Optional Cloudflare Turnstile token for protected deployments.
    pub cf_token: Option<String>,
    /// Optional random-subdomain flag for deployments that support it.
    pub enable_random_subdomain: Option<bool>,
}

impl CloudflareTempMailContext {
    /// Builds validated HTTP configuration for the Cloudflare worker client.
    pub fn api_config(&self) -> TempMailResult<ApiConfig> {
        ApiConfig::try_from(self)
    }

    /// Creates a Cloudflare worker client from this context.
    pub fn create_api(&self) -> TempMailResult<CloudflareTempMailApi> {
        CloudflareTempMailApi::new(self.api_config()?)
    }

    /// Builds a mailbox creation request from the context defaults.
    #[must_use]
    pub fn create_mailbox_request(&self) -> CreateMailboxRequest {
        CreateMailboxRequest::from(self)
    }

    /// Builds a raw `/api/new_address` request from the context defaults.
    #[must_use]
    pub fn new_address_request(&self) -> NewAddressRequest {
        NewAddressRequest::from(self)
    }
}

impl TryFrom<&CloudflareTempMailContext> for ApiConfig {
    type Error = crate::TempMailError;

    fn try_from(value: &CloudflareTempMailContext) -> Result<Self, Self::Error> {
        let mut builder = ApiConfig::builder(required_non_blank(&value.base_url, "base_url")?);
        if let Some(custom_auth) = trim_non_blank(value.custom_auth.as_deref()) {
            builder = builder.default_header("x-custom-auth", custom_auth);
        }
        builder.build()
    }
}

impl From<&CloudflareTempMailContext> for CreateMailboxRequest {
    fn from(value: &CloudflareTempMailContext) -> Self {
        let mut request = CreateMailboxRequest::random();
        request.name = trim_non_blank(value.address_name.as_deref()).map(str::to_owned);
        request.domain = trim_non_blank(value.address_domain.as_deref()).map(str::to_owned);
        request.cf_token = trim_non_blank(value.cf_token.as_deref()).map(str::to_owned);
        request.enable_random_subdomain = value.enable_random_subdomain.unwrap_or(false);
        request
    }
}

impl From<&CloudflareTempMailContext> for NewAddressRequest {
    fn from(value: &CloudflareTempMailContext) -> Self {
        Self {
            name: trim_non_blank(value.address_name.as_deref()).map(str::to_owned),
            domain: trim_non_blank(value.address_domain.as_deref()).map(str::to_owned),
            cf_token: trim_non_blank(value.cf_token.as_deref()).map(str::to_owned),
            enable_random_subdomain: value.enable_random_subdomain,
        }
    }
}
