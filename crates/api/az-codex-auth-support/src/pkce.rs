use crate::random::random_bytes;
use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use sha2::{Digest, Sha256};

/// RFC 7636 PKCE material for an OAuth authorization-code flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// Generates a high-entropy OAuth state value.
pub fn generate_state() -> CodexAuthSupportResult<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes::<32>()?))
}

/// Generates a PKCE verifier and S256 challenge.
pub fn generate_pkce_pair() -> CodexAuthSupportResult<PkcePair> {
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes::<64>()?);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(digest);

    Ok(PkcePair {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256".to_owned(),
    })
}

/// Builds a standard OAuth authorize URL using PKCE parameters.
pub fn build_authorize_url(
    issuer: impl AsRef<str>,
    client_id: impl AsRef<str>,
    redirect_uri: impl AsRef<str>,
    scope: impl AsRef<str>,
    state: impl AsRef<str>,
    pkce: &PkcePair,
) -> CodexAuthSupportResult<String> {
    let mut url = Url::parse(issuer.as_ref())
        .map_err(|_| CodexAuthSupportError::InvalidBaseUrl(issuer.as_ref().to_owned()))?
        .join("/oauth/authorize")
        .map_err(|_| CodexAuthSupportError::InvalidPath("/oauth/authorize".to_owned()))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id.as_ref())
        .append_pair("redirect_uri", redirect_uri.as_ref())
        .append_pair("scope", scope.as_ref())
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", &pkce.code_challenge_method)
        .append_pair("state", state.as_ref());

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_authorize_url, generate_pkce_pair};

    #[test]
    fn generate_pkce_pair_returns_url_safe_values() {
        let pair = generate_pkce_pair().expect("pkce generation");

        assert!(pair.code_verifier.len() >= 43);
        assert!(!pair.code_verifier.contains('='));
        assert!(!pair.code_challenge.contains('='));
        assert_eq!(pair.code_challenge_method, "S256");
    }

    #[test]
    fn build_authorize_url_adds_oauth_parameters() {
        let pair = generate_pkce_pair().expect("pkce generation");
        let url = build_authorize_url(
            "https://auth.example.com",
            "client-1",
            "http://localhost:1455/auth/callback",
            "openid offline_access",
            "state-1",
            &pair,
        )
        .expect("authorize url");

        assert!(url.starts_with("https://auth.example.com/oauth/authorize?"));
        assert!(url.contains("client_id=client-1"));
        assert!(url.contains("code_challenge_method=S256"));
    }
}
