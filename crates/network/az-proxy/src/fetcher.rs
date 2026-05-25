use crate::parser::parse_subscription;
use crate::types::{ProxyResult, ProxyNode};
use az_derive_aliases::{apply, plain_eq};
use reqwest::header::CONTENT_TYPE;

/// Raw subscription response fetched over HTTP.
#[apply(plain_eq)]
pub struct FetchedSubscription {
    /// Response body as text.
    pub body: String,
    /// Response `content-type` header when the server provided one.
    pub content_type: Option<String>,
}

/// Fetches subscription content from `url` with HTTP GET.
///
/// # Errors
///
/// Returns [`crate::types::ProxyError::Http`] when the request fails, the response
/// status is not successful, or the response body cannot be decoded as text.
pub async fn fetch_subscription(url: &str) -> ProxyResult<FetchedSubscription> {
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = response.text().await?;

    tracing::debug!(
        url,
        content_type = content_type.as_deref().unwrap_or(""),
        bytes = body.len(),
        "fetched proxy subscription"
    );

    Ok(FetchedSubscription { body, content_type })
}

/// Fetches a subscription URL and parses all supported proxy nodes.
///
/// The parser auto-detects Clash YAML, direct URI lists, and base64-encoded URI
/// lists. It also uses the HTTP `content-type` header as a parsing hint.
///
/// # Errors
///
/// Returns an error when fetching fails or the response does not contain any
/// usable supported proxy nodes.
pub async fn fetch_and_parse(url: &str) -> ProxyResult<Vec<ProxyNode>> {
    let fetched = fetch_subscription(url).await?;
    parse_subscription(&fetched.body, fetched.content_type.as_deref())
}
