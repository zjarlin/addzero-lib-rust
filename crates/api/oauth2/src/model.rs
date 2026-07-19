use anyhow::bail;
use serde_json::Value;

/// Successful OAuth2 token material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuth2TokenSuccess {
    /// Access token string.
    pub access_token: String,
    /// Optional refresh token.
    pub refresh_token: Option<String>,
    /// Optional OpenID Connect id token.
    pub id_token: Option<String>,
    /// Token type, usually `Bearer`.
    pub token_type: Option<String>,
    /// Lifetime in seconds.
    pub expires_in: Option<u64>,
    /// Scope string returned by the provider.
    pub scope: Option<String>,
}

/// OAuth2 token endpoint response, including structured provider errors.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OAuth2TokenResponse {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl OAuth2TokenResponse {
    /// Returns true when this response contains an access token.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.access_token
            .as_deref()
            .is_some_and(|token| !token.trim().is_empty())
    }

    /// Returns the access token or maps the provider error to a typed error.
    pub fn require_access_token(&self) -> anyhow::Result<&str> {
        if let Some(token) = self
            .access_token
            .as_deref()
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            return Ok(token);
        }

        if let Some(error) = self.error.as_deref() {
            let description = self.error_description.clone().unwrap_or_default();
            bail!("oauth provider error `{error}`: {description}");
        }

        bail!("invalid response: token response did not include access_token")
    }

    pub(crate) fn into_success(self) -> anyhow::Result<Self> {
        self.require_access_token()?;
        Ok(self)
    }

    /// Converts the response to a compact success-only shape.
    pub fn into_token_success(self) -> anyhow::Result<OAuth2TokenSuccess> {
        let access_token = self.require_access_token()?.to_owned();
        Ok(OAuth2TokenSuccess {
            access_token,
            refresh_token: self.refresh_token,
            id_token: self.id_token,
            token_type: self.token_type,
            expires_in: self.expires_in,
            scope: self.scope,
        })
    }
}

/// Device authorization response from an OAuth2 provider.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OAuth2DeviceAuthorization {
    #[serde(default)]
    pub device_code: String,
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub interval: Option<u64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl OAuth2DeviceAuthorization {
    /// Returns the best URL to open for user authorization.
    #[must_use]
    pub fn browser_verification_url(&self) -> &str {
        self.verification_uri_complete
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&self.verification_uri)
    }
}

/// Result of one device token polling request.
#[derive(Clone, Debug, PartialEq)]
pub enum OAuth2DeviceTokenPoll {
    /// User authorization is still pending.
    Pending,
    /// Provider requested slower polling.
    SlowDown {
        /// Next poll interval in seconds.
        next_interval_secs: u64,
    },
    /// Device code expired.
    Expired {
        /// Human-readable reason.
        message: String,
    },
    /// User denied access.
    AccessDenied {
        /// Human-readable reason.
        message: String,
    },
    /// Provider returned an unexpected OAuth error.
    Error {
        /// Human-readable reason.
        message: String,
        /// Raw token endpoint response.
        response: OAuth2TokenResponse,
    },
    /// Token exchange completed.
    Success(OAuth2TokenResponse),
}

impl OAuth2DeviceTokenPoll {
    /// Returns true for terminal device-flow states.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Success(_)
                | Self::Expired { .. }
                | Self::AccessDenied { .. }
                | Self::Error { .. }
        )
    }
}
