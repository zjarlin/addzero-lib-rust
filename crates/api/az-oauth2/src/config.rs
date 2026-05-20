use crate::{OAuth2Error, OAuth2Result, PkcePair};
use std::collections::BTreeMap;
use std::time::Duration;

const DEFAULT_USER_AGENT: &str = "az-oauth2/2026.5";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;
const DEFAULT_LOOPBACK_BIND_ADDR: &str = "127.0.0.1:0";
const DEFAULT_LOOPBACK_PATH: &str = "/oauth/callback";

/// OAuth2 endpoint and client configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2Config {
    /// Authorization endpoint URL.
    pub authorization_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Optional device authorization endpoint URL.
    pub device_authorization_url: Option<String>,
    /// OAuth client id.
    pub client_id: String,
    /// Optional client secret for confidential clients or providers that still issue one.
    pub client_secret: Option<String>,
    /// Default redirect URI used by authorization-code flows.
    pub redirect_uri: Option<String>,
    /// Default OAuth scopes.
    pub scopes: Vec<String>,
    /// HTTP user agent sent by the client.
    pub user_agent: Option<String>,
    /// TCP connect timeout.
    pub connect_timeout: Duration,
    /// Whole request timeout.
    pub request_timeout: Duration,
}

impl OAuth2Config {
    /// Starts a config builder with required OAuth endpoints and client id.
    pub fn builder(
        authorization_url: impl Into<String>,
        token_url: impl Into<String>,
        client_id: impl Into<String>,
    ) -> OAuth2ConfigBuilder {
        OAuth2ConfigBuilder {
            config: Self {
                authorization_url: authorization_url.into(),
                token_url: token_url.into(),
                device_authorization_url: None,
                client_id: client_id.into(),
                client_secret: None,
                redirect_uri: None,
                scopes: Vec::new(),
                user_agent: Some(DEFAULT_USER_AGENT.to_owned()),
                connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
                request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            },
        }
    }

    pub(crate) fn validate(&self) -> OAuth2Result<()> {
        if self.authorization_url.trim().is_empty() {
            return Err(OAuth2Error::InvalidConfig(
                "authorization_url cannot be blank".to_owned(),
            ));
        }
        if self.token_url.trim().is_empty() {
            return Err(OAuth2Error::InvalidConfig(
                "token_url cannot be blank".to_owned(),
            ));
        }
        if self.client_id.trim().is_empty() {
            return Err(OAuth2Error::InvalidConfig(
                "client_id cannot be blank".to_owned(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(OAuth2Error::InvalidConfig(
                "connect_timeout must be greater than zero".to_owned(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(OAuth2Error::InvalidConfig(
                "request_timeout must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Builder for [`OAuth2Config`].
#[derive(Debug, Clone)]
pub struct OAuth2ConfigBuilder {
    pub(crate) config: OAuth2Config,
}

impl OAuth2ConfigBuilder {
    /// Sets the optional device authorization endpoint.
    #[must_use]
    pub fn device_authorization_url(mut self, value: impl Into<String>) -> Self {
        self.config.device_authorization_url = Some(value.into());
        self
    }

    /// Sets the optional client secret.
    #[must_use]
    pub fn client_secret(mut self, value: impl Into<String>) -> Self {
        self.config.client_secret = Some(value.into());
        self
    }

    /// Sets the default redirect URI.
    #[must_use]
    pub fn redirect_uri(mut self, value: impl Into<String>) -> Self {
        self.config.redirect_uri = Some(value.into());
        self
    }

    /// Adds one OAuth scope.
    #[must_use]
    pub fn scope(mut self, value: impl Into<String>) -> Self {
        self.config.scopes.push(value.into());
        self
    }

    /// Adds multiple OAuth scopes.
    #[must_use]
    pub fn scopes<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config
            .scopes
            .extend(values.into_iter().map(Into::into));
        self
    }

    /// Overrides or clears the HTTP user agent.
    #[must_use]
    pub fn user_agent(mut self, value: Option<impl Into<String>>) -> Self {
        self.config.user_agent = value.map(Into::into);
        self
    }

    /// Sets TCP connect timeout.
    #[must_use]
    pub const fn connect_timeout(mut self, value: Duration) -> Self {
        self.config.connect_timeout = value;
        self
    }

    /// Sets whole request timeout.
    #[must_use]
    pub const fn request_timeout(mut self, value: Duration) -> Self {
        self.config.request_timeout = value;
        self
    }

    /// Validates and returns the config.
    pub fn build(self) -> OAuth2Result<OAuth2Config> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Options for one authorization-code request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationCodeOptions {
    /// Redirect URI for this request. If omitted, the config default is used.
    pub redirect_uri: Option<String>,
    /// Scopes for this request. If empty, config scopes are used.
    pub scopes: Vec<String>,
    /// OAuth state. If omitted, a random state is generated.
    pub state: Option<String>,
    /// PKCE pair. If omitted, a new S256 pair is generated.
    pub pkce: Option<PkcePair>,
    /// Optional `login_hint`.
    pub login_hint: Option<String>,
    /// Optional provider-specific `access_type`, useful for Google `offline`.
    pub access_type: Option<String>,
    /// Optional provider-specific `prompt`, useful for Google `consent`.
    pub prompt: Option<String>,
    /// Extra authorization query parameters.
    pub extra_params: BTreeMap<String, String>,
    /// Loopback bind address used by [`crate::OAuth2Client::begin_loopback_authorization`].
    pub loopback_bind_addr: String,
    /// Loopback callback path.
    pub loopback_path: String,
}

impl AuthorizationCodeOptions {
    /// Creates default authorization options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the redirect URI.
    #[must_use]
    pub fn redirect_uri(mut self, value: impl Into<String>) -> Self {
        self.redirect_uri = Some(value.into());
        self
    }

    /// Adds one scope for this request.
    #[must_use]
    pub fn scope(mut self, value: impl Into<String>) -> Self {
        self.scopes.push(value.into());
        self
    }

    /// Adds multiple scopes for this request.
    #[must_use]
    pub fn scopes<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes.extend(values.into_iter().map(Into::into));
        self
    }

    /// Sets OAuth state.
    #[must_use]
    pub fn state(mut self, value: impl Into<String>) -> Self {
        self.state = Some(value.into());
        self
    }

    /// Sets explicit PKCE material, mostly for deterministic tests.
    #[must_use]
    pub fn pkce(mut self, value: PkcePair) -> Self {
        self.pkce = Some(value);
        self
    }

    /// Sets `login_hint`.
    #[must_use]
    pub fn login_hint(mut self, value: impl Into<String>) -> Self {
        self.login_hint = Some(value.into());
        self
    }

    /// Sets `access_type`.
    #[must_use]
    pub fn access_type(mut self, value: impl Into<String>) -> Self {
        self.access_type = Some(value.into());
        self
    }

    /// Sets `prompt`.
    #[must_use]
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    /// Adds an extra authorization parameter.
    #[must_use]
    pub fn extra_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.insert(name.into(), value.into());
        self
    }

    /// Sets loopback bind address, for example `127.0.0.1:0`.
    #[must_use]
    pub fn loopback_bind_addr(mut self, value: impl Into<String>) -> Self {
        self.loopback_bind_addr = value.into();
        self
    }

    /// Sets loopback callback path.
    #[must_use]
    pub fn loopback_path(mut self, value: impl Into<String>) -> Self {
        self.loopback_path = value.into();
        self
    }
}

impl Default for AuthorizationCodeOptions {
    fn default() -> Self {
        Self {
            redirect_uri: None,
            scopes: Vec::new(),
            state: None,
            pkce: None,
            login_hint: None,
            access_type: None,
            prompt: None,
            extra_params: BTreeMap::new(),
            loopback_bind_addr: DEFAULT_LOOPBACK_BIND_ADDR.to_owned(),
            loopback_path: DEFAULT_LOOPBACK_PATH.to_owned(),
        }
    }
}
