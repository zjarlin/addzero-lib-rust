use crate::random::random_bytes;
use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use sha2::{Digest, Sha256};

/// OAuth 授权码流程使用的 RFC 7636 PKCE 材料。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PkcePair {
    /// OAuth PKCE code verifier。
    pub code_verifier: String,
    /// S256 code challenge。
    pub code_challenge: String,
    /// code challenge method，当前固定为 `S256`。
    pub code_challenge_method: String,
}

/// 生成高熵 OAuth state 值。
pub fn generate_state() -> anyhow::Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes::<32>()?))
}

/// 生成 PKCE verifier 和 S256 challenge。
pub fn generate_pkce_pair() -> anyhow::Result<PkcePair> {
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes::<64>()?);
    let digest = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(digest);

    Ok(PkcePair {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256".to_owned(),
    })
}

/// 使用 PKCE 参数构建标准 OAuth authorize URL。
pub fn build_authorize_url(
    issuer: impl AsRef<str>,
    client_id: impl AsRef<str>,
    redirect_uri: impl AsRef<str>,
    scope: impl AsRef<str>,
    state: impl AsRef<str>,
    pkce: &PkcePair,
) -> anyhow::Result<String> {
    let mut url = Url::parse(issuer.as_ref())
        .with_context(|| format!("invalid base url `{}`", issuer.as_ref()))?
        .join("/oauth/authorize")
        .context("invalid request path `/oauth/authorize`")?;

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
