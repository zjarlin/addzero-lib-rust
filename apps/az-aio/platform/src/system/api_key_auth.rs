//! 系统级 API Key 认证中间件。
//!
//! 该层只在调用方显式携带 `X-API-Key`、`api_key` 查询参数或
//! `Authorization: Bearer az_live_*` 时校验密钥；未携带密钥的本地后台页面仍保持可用。

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    core::api_error::ApiError,
    system::{model::SystemApiKeySummary, store::SystemAdminStore},
};

#[derive(Clone)]
pub struct SystemApiKeyAuthState {
    store: Option<SystemAdminStore>,
}

impl SystemApiKeyAuthState {
    pub fn degraded() -> Self {
        Self { store: None }
    }

    pub fn from_store(store: Option<SystemAdminStore>) -> Self {
        Self { store }
    }

    async fn authorize(&self, api_key: &str) -> anyhow::Result<Option<SystemApiKeySummary>> {
        let Some(store) = &self.store else {
            anyhow::bail!("missing system-admin database url");
        };
        store.authorize_api_key(api_key).await
    }
}

/// Optional API key boundary for the native API surface.
///
/// A valid key is inserted into request extensions as [`SystemApiKeySummary`]. An invalid explicitly
/// supplied key fails fast with 401 so external callers cannot silently fall through.
pub async fn optional_system_api_key_auth(
    State(state): State<SystemApiKeyAuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(api_key) = api_key_from_request(request.headers(), request.uri().query()) else {
        return next.run(request).await;
    };

    match state.authorize(&api_key).await {
        Ok(Some(summary)) => {
            request.extensions_mut().insert(summary);
            next.run(request).await
        }
        Ok(None) => ApiError::new(StatusCode::UNAUTHORIZED, "invalid api_key").into_response(),
        Err(error) => ApiError::from(error).into_response(),
    }
}

fn api_key_from_request(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    header_value(headers, "x-api-key")
        .or_else(|| bearer_api_key(headers))
        .or_else(|| query_api_key(query))
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bearer_api_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| value.starts_with("az_live_"))
        .map(ToOwned::to_owned)
}

fn query_api_key(query: Option<&str>) -> Option<String> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find_map(|(key, value)| {
            if key == "api_key" {
                urlencoding::decode(value)
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn api_key_prefers_explicit_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("az_live_header"));

        let key = api_key_from_request(&headers, Some("api_key=az_live_query"));

        // 关键断言：显式 API Key header 不会被查询参数覆盖。
        assert_eq!(key.as_deref(), Some("az_live_header"));
    }

    #[test]
    fn bearer_only_treats_system_api_key_prefix_as_global_key() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer edge-demo-weather-token"),
        );

        let key = api_key_from_request(&headers, None);

        // 关键断言：不能拦截 edge-gateway 既有 Bearer token。
        assert_eq!(key, None);
    }

    #[test]
    fn query_supports_api_key_parameter() {
        let headers = HeaderMap::new();

        let key = api_key_from_request(&headers, Some("route=/demo&api_key=az_live_query"));

        assert_eq!(key.as_deref(), Some("az_live_query"));
    }
}
