use crate::{CodexAuthSupportError, CodexAuthSupportResult};
use az_derive_aliases::{apply, plain_eq, serde_eq, serde_eq_default};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// OAuth token response fields needed to write a Codex auth file.
#[apply(serde_eq_default)]
pub struct OAuthTokens {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
}

/// CLIProxyAPI-compatible Codex auth-file content.
#[apply(serde_eq)]
pub struct CodexAuthFile {
    #[serde(rename = "type")]
    pub kind: String,
    pub email: String,
    pub expired: String,
    pub id_token: String,
    pub account_id: String,
    pub access_token: String,
    pub last_refresh: String,
    pub refresh_token: String,
}

impl CodexAuthFile {
    /// Builds a Codex auth file from OAuth tokens using the current time.
    pub fn from_tokens(email: impl Into<String>, tokens: OAuthTokens) -> Self {
        let offset = FixedOffset::east_opt(8 * 60 * 60).expect("valid +08:00 offset");
        Self::from_tokens_at(email, tokens, Utc::now(), offset)
    }

    /// Builds a Codex auth file from OAuth tokens at a deterministic timestamp.
    pub fn from_tokens_at(
        email: impl Into<String>,
        tokens: OAuthTokens,
        now: DateTime<Utc>,
        offset: FixedOffset,
    ) -> Self {
        let payload = decode_jwt_payload(&tokens.access_token).ok();
        let account_id = payload
            .as_ref()
            .and_then(|payload| payload.get("https://api.openai.com/auth"))
            .and_then(|auth| auth.get("chatgpt_account_id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        let expired = payload
            .as_ref()
            .and_then(|payload| payload.get("exp"))
            .and_then(Value::as_i64)
            .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
            .map(|value| {
                value
                    .with_timezone(&offset)
                    .to_rfc3339_opts(SecondsFormat::Secs, false)
            })
            .unwrap_or_default();

        Self {
            kind: "codex".to_owned(),
            email: email.into(),
            expired,
            id_token: tokens.id_token,
            account_id,
            access_token: tokens.access_token,
            last_refresh: now
                .with_timezone(&offset)
                .to_rfc3339_opts(SecondsFormat::Secs, false),
            refresh_token: tokens.refresh_token,
        }
    }

    /// Writes this auth file to `dir/<safe email>.json`.
    pub fn write_to_dir(
        &self,
        dir: impl AsRef<Path>,
    ) -> CodexAuthSupportResult<AuthFileWriteOutcome> {
        fs::create_dir_all(dir.as_ref())?;
        let path = dir.as_ref().join(safe_auth_filename(&self.email));
        let payload = serde_json::to_vec_pretty(self)?;
        fs::write(&path, payload)?;
        Ok(AuthFileWriteOutcome { path })
    }
}

/// Result metadata for an auth-file write.
#[apply(plain_eq)]
pub struct AuthFileWriteOutcome {
    pub path: PathBuf,
}

/// Decodes a JWT payload without verifying its signature.
///
/// Use this only for non-authoritative metadata extraction, such as deriving
/// `exp` or account identifiers for local auth-file labels.
pub fn decode_jwt_payload(token: &str) -> CodexAuthSupportResult<Value> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(CodexAuthSupportError::InvalidToken(
            "JWT should contain three dot-separated segments".to_owned(),
        ));
    }

    let mut payload = parts[1].to_owned();
    while payload.len() % 4 != 0 {
        payload.push('=');
    }

    let bytes = URL_SAFE
        .decode(payload)
        .map_err(|error| CodexAuthSupportError::InvalidToken(error.to_string()))?;
    Ok(serde_json::from_slice(bytes.as_ref())?)
}

/// Creates a filesystem-safe auth-file name while preserving readable email labels.
pub fn safe_auth_filename(email: impl AsRef<str>) -> String {
    let mut stem = email
        .as_ref()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '@' | '.' | '_' | '-' | '+' => ch,
            _ => '_',
        })
        .collect::<String>();

    if stem.trim_matches('_').is_empty() {
        stem = "codex-auth".to_owned();
    }

    if !stem.ends_with(".json") {
        stem.push_str(".json");
    }

    stem
}

#[cfg(test)]
mod tests {
    use super::{CodexAuthFile, OAuthTokens, decode_jwt_payload, safe_auth_filename};
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::{FixedOffset, TimeZone, Utc};
    use serde_json::{Value, json};

    #[test]
    fn decode_jwt_payload_reads_json_payload() {
        let token = fake_jwt(json!({"sub":"user-1"}));
        let payload = decode_jwt_payload(&token).expect("jwt payload");

        assert_eq!(payload.get("sub").and_then(Value::as_str), Some("user-1"));
    }

    #[test]
    fn codex_auth_file_extracts_expiry_and_account_id() {
        let access_token = fake_jwt(json!({
            "exp": 1_800_000_000,
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "account-1"
            }
        }));
        let now = Utc.with_ymd_and_hms(2026, 5, 9, 1, 2, 3).unwrap();
        let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap();

        let file = CodexAuthFile::from_tokens_at(
            "a@example.com",
            OAuthTokens {
                access_token,
                refresh_token: "refresh".to_owned(),
                id_token: "id".to_owned(),
            },
            now,
            offset,
        );

        assert_eq!(file.account_id, "account-1");
        assert_eq!(file.last_refresh, "2026-05-09T09:02:03+08:00");
        assert!(file.expired.ends_with("+08:00"));
    }

    #[test]
    fn safe_auth_filename_blocks_path_segments() {
        assert_eq!(
            safe_auth_filename("../user@example.com"),
            ".._user@example.com.json"
        );
    }

    fn fake_jwt(payload: serde_json::Value) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
        let payload = URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("{header}.{payload}.signature")
    }
}
