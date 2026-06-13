use crate::config::OAuth2Config;
use az_derive_aliases::{apply, plain_default_copy_eq};

const GOOGLE_AUTHORIZATION_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_DEVICE_AUTHORIZATION_URL: &str = "https://oauth2.googleapis.com/device/code";

/// Google OAuth2 endpoints and scope constants.
#[apply(plain_default_copy_eq)]
pub struct GoogleOAuth2;

impl GoogleOAuth2 {
    /// Gmail read-only scope for reading mailbox messages without modifying them.
    pub const GMAIL_READONLY_SCOPE: &'static str = "https://www.googleapis.com/auth/gmail.readonly";
    /// Google OAuth2 authorization endpoint.
    pub const AUTHORIZATION_URL: &'static str = GOOGLE_AUTHORIZATION_URL;
    /// Google OAuth2 token endpoint.
    pub const TOKEN_URL: &'static str = GOOGLE_TOKEN_URL;
    /// Google OAuth2 device authorization endpoint.
    pub const DEVICE_AUTHORIZATION_URL: &'static str = GOOGLE_DEVICE_AUTHORIZATION_URL;

    /// Creates a Google installed-app OAuth config builder.
    ///
    /// Desktop and CLI callers normally pair this with a loopback redirect and PKCE.
    pub fn installed_app(client_id: impl Into<String>) -> crate::config::OAuth2ConfigBuilder {
        OAuth2Config::builder(GOOGLE_AUTHORIZATION_URL, GOOGLE_TOKEN_URL, client_id)
            .device_authorization_url(GOOGLE_DEVICE_AUTHORIZATION_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::GoogleOAuth2;

    #[test]
    fn google_installed_app_sets_official_endpoints() {
        let config = GoogleOAuth2::installed_app("client-1")
            .scope(GoogleOAuth2::GMAIL_READONLY_SCOPE)
            .build()
            .expect("config");

        assert_eq!(
            config.authorization_url,
            "https://accounts.google.com/o/oauth2/v2/auth"
        );
        assert_eq!(config.token_url, "https://oauth2.googleapis.com/token");
        assert_eq!(
            config.device_authorization_url.as_deref(),
            Some("https://oauth2.googleapis.com/device/code")
        );
        assert_eq!(config.scopes, vec![GoogleOAuth2::GMAIL_READONLY_SCOPE]);
    }
}
