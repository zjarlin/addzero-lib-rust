use anyhow::{Context, anyhow, bail};
use az_str::sanitize::sanitize_file_name_with_extension;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use chrono::{DateTime, FixedOffset, Offset, SecondsFormat, Utc};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// 写入 Codex 认证文件所需的 OAuth token 响应字段。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OAuthTokens {
    /// OAuth access token。
    #[serde(default)]
    pub access_token: String,
    /// OAuth refresh token。
    #[serde(default)]
    pub refresh_token: String,
    /// OAuth id token。
    #[serde(default)]
    pub id_token: String,
}

/// 兼容 CLIProxyAPI 的 Codex 认证文件内容。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CodexAuthFile {
    /// 认证文件类型，当前固定为 `codex`。
    #[serde(rename = "type")]
    pub kind: String,
    /// 账号邮箱。
    pub email: String,
    /// access token 过期时间。
    pub expired: String,
    /// OAuth id token。
    pub id_token: String,
    /// ChatGPT 账号 ID。
    pub account_id: String,
    /// OAuth access token。
    pub access_token: String,
    /// 上次刷新时间。
    pub last_refresh: String,
    /// OAuth refresh token。
    pub refresh_token: String,
}

impl CodexAuthFile {
    /// 使用当前时间根据 OAuth token 构建 Codex 认证文件。
    pub fn from_tokens(email: impl Into<String>, tokens: OAuthTokens) -> Self {
        let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap_or_else(|| Utc.fix());
        Self::from_tokens_at(email, tokens, Utc::now(), offset)
    }

    /// 使用确定性时间戳根据 OAuth token 构建 Codex 认证文件。
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

    /// 将当前认证文件写入 `dir/<safe email>.json`。
    pub fn write_to_dir(
        &self,
        dir: impl AsRef<Path>,
    ) -> anyhow::Result<AuthFileWriteOutcome> {
        fs::create_dir_all(dir.as_ref()).with_context(|| {
            format!(
                "failed to create Codex auth directory `{}`",
                dir.as_ref().display()
            )
        })?;
        let path = dir.as_ref().join(safe_auth_filename(&self.email));
        let payload = serde_json::to_vec_pretty(self)
            .context("failed to serialize Codex auth file payload")?;
        fs::write(&path, payload)
            .with_context(|| format!("failed to write Codex auth file `{}`", path.display()))?;
        Ok(AuthFileWriteOutcome { path })
    }
}

/// 认证文件写入结果元数据。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthFileWriteOutcome {
    /// 实际写入的文件路径。
    pub path: PathBuf,
}

/// 解码 JWT payload，但不校验签名。
///
/// 该函数只应用于非权威元数据提取，例如为本地认证文件标签读取 `exp` 或账号标识。
pub fn decode_jwt_payload(token: &str) -> anyhow::Result<Value> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        bail!("invalid token: JWT should contain three dot-separated segments");
    }

    let mut payload = parts[1].to_owned();
    while payload.len() % 4 != 0 {
        payload.push('=');
    }

    let bytes = URL_SAFE
        .decode(payload)
        .map_err(|error| anyhow!("invalid token: {error}"))?;
    serde_json::from_slice(bytes.as_ref()).context("failed to parse JWT payload JSON")
}

/// 创建文件系统安全的认证文件名，同时保留可读邮箱标签。
pub fn safe_auth_filename(email: impl AsRef<str>) -> String {
    sanitize_file_name_with_extension(email.as_ref(), "@._-+", '_', "json", "codex-auth")
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
