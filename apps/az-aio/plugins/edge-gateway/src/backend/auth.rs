use std::{collections::BTreeSet, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Demo token used by the first callable weather asset.
pub const DEMO_WEATHER_TOKEN: &str = "edge-demo-weather-token";

/// API token record for sub2api-style edge asset calls.
///
/// The gateway keeps a token hash, allowed route scopes, status, and usage timestamps. This is the
/// minimal reusable mechanism needed before moving the same shape to PostgreSQL.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeApiToken {
    pub id: String,
    pub name: String,
    pub token_hash: String,
    pub allowed_routes: BTreeSet<String>,
    pub status: String,
    pub expires_at_epoch_secs: Option<u64>,
    pub last_used_at_epoch_secs: Option<u64>,
}

impl EdgeApiToken {
    /// Creates an active scoped token from cleartext token material.
    pub fn active(
        id: impl Into<String>,
        name: impl Into<String>,
        cleartext_token: &str,
        allowed_routes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            token_hash: token_hash(cleartext_token),
            allowed_routes: allowed_routes.into_iter().map(Into::into).collect(),
            status: "active".to_string(),
            expires_at_epoch_secs: None,
            last_used_at_epoch_secs: None,
        }
    }

    /// Returns whether this token can call a route now.
    pub fn allows_route(&self, route: &str, now_epoch_secs: u64) -> bool {
        self.status == "active"
            && self
                .expires_at_epoch_secs
                .is_none_or(|expires_at| expires_at > now_epoch_secs)
            && self.allowed_routes.contains(route)
    }
}

/// Authenticated caller identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeAuthorizedToken {
    pub token_id: String,
    pub token_name: String,
}

/// Append-only usage event for callable edge assets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeUsageRecord {
    pub token_id: String,
    pub route: String,
    pub asset_id: String,
    pub status_code: u16,
    pub request_units: u32,
    pub duration_ms: u128,
    pub created_at_epoch_secs: u64,
}

/// In-memory authorization and usage store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EdgeTokenStore {
    tokens: Vec<EdgeApiToken>,
    usage_records: Vec<EdgeUsageRecord>,
}

impl EdgeTokenStore {
    /// Adds or replaces a token.
    pub fn upsert_token(&mut self, token: EdgeApiToken) {
        if let Some(existing) = self.tokens.iter_mut().find(|existing| existing.id == token.id) {
            *existing = token;
            return;
        }
        self.tokens.push(token);
    }

    /// Authorizes a cleartext bearer token for a route.
    pub fn authorize(&mut self, route: &str, cleartext_token: &str) -> Result<EdgeAuthorizedToken, EdgeAuthError> {
        let now = now_epoch_secs();
        let hash = token_hash(cleartext_token);
        let Some(token) = self.tokens.iter_mut().find(|token| token.token_hash == hash) else {
            return Err(EdgeAuthError::InvalidToken);
        };
        if !token.allows_route(route, now) {
            return Err(EdgeAuthError::ForbiddenRoute);
        }
        token.last_used_at_epoch_secs = Some(now);
        Ok(EdgeAuthorizedToken {
            token_id: token.id.clone(),
            token_name: token.name.clone(),
        })
    }

    /// Records one callable asset usage event.
    pub fn record_usage(&mut self, record: EdgeUsageRecord) {
        self.usage_records.push(record);
    }

    /// Returns usage records for UI/API diagnostics.
    pub fn usage_records(&self) -> &[EdgeUsageRecord] {
        &self.usage_records
    }
}

/// Authorization failure cases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeAuthError {
    MissingToken,
    InvalidToken,
    ForbiddenRoute,
}

impl EdgeAuthError {
    /// HTTP status for the auth failure.
    pub const fn status_code(self) -> u16 {
        match self {
            Self::MissingToken | Self::InvalidToken => 401,
            Self::ForbiddenRoute => 403,
        }
    }

    /// Machine-readable error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingToken => "missing_token",
            Self::InvalidToken => "invalid_token",
            Self::ForbiddenRoute => "forbidden_route",
        }
    }
}

/// Extracts a Bearer token from an Authorization header value.
pub fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    authorization
        .map(str::trim)
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

/// Returns a SHA-256 token hash for storage/comparison.
pub fn token_hash(cleartext_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cleartext_token.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Current Unix timestamp in seconds.
pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{bearer_token, EdgeApiToken, EdgeAuthError, EdgeTokenStore};

    #[test]
    fn bearer_token_extracts_authorization_value() {
        assert_eq!(bearer_token(Some("Bearer edge-demo-weather-token")), Some("edge-demo-weather-token"));
    }

    #[test]
    fn token_store_authorizes_scoped_token() {
        let mut store = EdgeTokenStore::default();
        store.upsert_token(EdgeApiToken::active(
            "tok_weather_demo",
            "demo weather",
            "secret",
            ["/api/edge-gateway/assets/weather/current"],
        ));

        let authorized = store
            .authorize("/api/edge-gateway/assets/weather/current", "secret")
            .unwrap();

        assert_eq!(authorized.token_id, "tok_weather_demo");
    }

    #[test]
    fn token_store_rejects_wrong_route() {
        let mut store = EdgeTokenStore::default();
        store.upsert_token(EdgeApiToken::active(
            "tok_weather_demo",
            "demo weather",
            "secret",
            ["/api/edge-gateway/assets/weather/current"],
        ));

        let error = store.authorize("/api/edge-gateway/other", "secret").unwrap_err();

        assert_eq!(error, EdgeAuthError::ForbiddenRoute);
    }
}
