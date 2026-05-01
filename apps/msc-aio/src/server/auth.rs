use std::time::Duration;

use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use addzero_agent_runtime_contract::{LoginRequest, SessionUser};

type HmacSha256 = Hmac<Sha256>;

const COOKIE_NAME: &str = "dioxus_admin_session";

#[derive(Clone)]
pub struct AdminSessionService {
    username: String,
    password: String,
    signing_key: Vec<u8>,
    ttl: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminAuthFailure {
    UsernameNotFound,
    PasswordIncorrect,
}

impl AdminAuthFailure {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UsernameNotFound => "用户名不存在",
            Self::PasswordIncorrect => "密码错误",
        }
    }
}

impl AdminSessionService {
    pub fn from_env() -> Self {
        let username = std::env::var("DIOXUS_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let password = std::env::var("DIOXUS_ADMIN_PASSWORD").unwrap_or_else(|_| "admin".into());
        let secret = std::env::var("DIOXUS_ADMIN_SESSION_SECRET")
            .unwrap_or_else(|_| "dev-session-secret-change-me".into());
        if secret == "dev-session-secret-change-me" {
            log::warn!(
                "⚠️  DIOXUS_ADMIN_SESSION_SECRET 未设置，使用默认密钥。生产环境请务必配置！"
            );
        }
        if username == "admin" && password == "admin" {
            log::warn!(
                "⚠️  使用默认凭证 admin/admin，生产环境请配置 DIOXUS_ADMIN_USERNAME 和 DIOXUS_ADMIN_PASSWORD"
            );
        }
        let ttl_hours = std::env::var("DIOXUS_ADMIN_SESSION_HOURS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(12);

        Self {
            username,
            password,
            signing_key: secret.into_bytes(),
            ttl: Duration::from_secs(ttl_hours.saturating_mul(3600)),
        }
    }

    pub fn authenticate(&self, input: &LoginRequest) -> Result<String, AdminAuthFailure> {
        self.validate_credentials(input)?;
        self.issue_cookie(input.username.trim())
            .map_err(|_| AdminAuthFailure::PasswordIncorrect)
    }

    pub fn current_user(&self, headers: &HeaderMap) -> Option<String> {
        let raw_cookie = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
        let session_value = find_cookie(raw_cookie, COOKIE_NAME)?;
        self.verify_cookie(session_value)
    }

    pub fn session_user(&self, headers: &HeaderMap) -> SessionUser {
        match self.current_user(headers) {
            Some(username) => SessionUser {
                authenticated: true,
                username: Some(username),
            },
            None => SessionUser {
                authenticated: false,
                username: None,
            },
        }
    }

    pub fn set_cookie_header(&self, cookie_value: &str) -> String {
        format!(
            "{COOKIE_NAME}={cookie_value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            self.ttl.as_secs()
        )
    }

    pub fn clear_cookie_header(&self) -> String {
        format!("{COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
    }

    pub fn validate_credentials(&self, input: &LoginRequest) -> Result<(), AdminAuthFailure> {
        if input.username.trim() != self.username {
            return Err(AdminAuthFailure::UsernameNotFound);
        }
        if input.password != self.password {
            return Err(AdminAuthFailure::PasswordIncorrect);
        }
        Ok(())
    }

    fn issue_cookie(&self, username: &str) -> Result<String, anyhow::Error> {
        let expires_at = Utc::now()
            + chrono::Duration::from_std(self.ttl).unwrap_or_else(|_| chrono::Duration::hours(12));
        let payload = format!("{}|{}", username, expires_at.timestamp());
        let payload_encoded = URL_SAFE_NO_PAD.encode(payload.as_bytes());
        let signature = sign(&self.signing_key, payload.as_bytes())?;
        Ok(format!(
            "{}.{}",
            payload_encoded,
            URL_SAFE_NO_PAD.encode(signature)
        ))
    }

    fn verify_cookie(&self, cookie_value: &str) -> Option<String> {
        let (payload_encoded, signature_encoded) = cookie_value.split_once('.')?;
        let payload = URL_SAFE_NO_PAD.decode(payload_encoded).ok()?;
        let signature = URL_SAFE_NO_PAD.decode(signature_encoded).ok()?;
        let expected = sign(&self.signing_key, &payload).ok()?;
        if expected != signature {
            return None;
        }

        let payload = String::from_utf8(payload).ok()?;
        let (username, ts) = payload.split_once('|')?;
        let expires_at = ts.parse::<i64>().ok()?;
        let expires_at = DateTime::<Utc>::from_timestamp(expires_at, 0)?;
        if expires_at < Utc::now() {
            return None;
        }
        Some(username.to_string())
    }
}

fn sign(key: &[u8], payload: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut mac =
        HmacSha256::new_from_slice(key).map_err(|err| anyhow::anyhow!("HMAC 初始化失败: {err}"))?;
    mac.update(payload);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn find_cookie<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    header.split(';').map(str::trim).find_map(|entry| {
        entry
            .split_once('=')
            .and_then(|(name, value)| if name == key { Some(value) } else { None })
    })
}

#[cfg(test)]
mod tests {
    use super::AdminSessionService;
    use addzero_agent_runtime_contract::LoginRequest;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn cookie_round_trip_should_restore_user() {
        let service = AdminSessionService::from_env();
        let cookie = service
            .authenticate(&LoginRequest {
                username: "admin".into(),
                password: "admin".into(),
            })
            .expect("default credentials should authenticate");
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_str(&format!("dioxus_admin_session={cookie}"))
                .expect("cookie header should be valid"),
        );

        assert_eq!(service.current_user(&headers).as_deref(), Some("admin"));
    }
}
