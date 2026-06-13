use crate::random::random_bytes;
use az_derive_aliases::{apply, plain_eq};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// RFC 7636 PKCE verifier/challenge pair.
#[apply(plain_eq)]
pub struct PkcePair {
    /// High-entropy value sent only to the token endpoint.
    pub code_verifier: String,
    /// S256 challenge sent to the authorization endpoint.
    pub code_challenge: String,
    /// Challenge method, normally `S256`.
    pub code_challenge_method: String,
}

/// Generates a high-entropy OAuth state value.
pub fn generate_state() -> anyhow::Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(random_bytes::<32>()?))
}

/// Generates an RFC 7636 S256 PKCE pair.
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

#[cfg(test)]
mod tests {
    use super::{generate_pkce_pair, generate_state};

    #[test]
    fn generated_pkce_values_are_url_safe() {
        let pair = generate_pkce_pair().expect("pkce pair");

        assert!(pair.code_verifier.len() >= 43);
        assert!(!pair.code_verifier.contains('='));
        assert!(!pair.code_challenge.contains('='));
        assert_eq!(pair.code_challenge_method, "S256");
    }

    #[test]
    fn generated_state_has_entropy() {
        let one = generate_state().expect("state");
        let two = generate_state().expect("state");

        assert_ne!(one, two);
        assert!(one.len() >= 32);
    }
}
