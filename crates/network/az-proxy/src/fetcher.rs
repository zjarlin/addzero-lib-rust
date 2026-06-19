use crate::parser::parse_subscription;
use crate::types::ProxyNode;
use anyhow::{Context, Result};
use reqwest::header::CONTENT_TYPE;

/// 通过 HTTP 获取到的原始订阅响应。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FetchedSubscription {
    /// 响应正文文本。
    pub body: String,
    /// 服务端提供的 `content-type` 响应头。
    pub content_type: Option<String>,
}

/// 通过 HTTP GET 从 `url` 获取订阅内容。
///
/// # Errors
///
/// 当请求失败、响应状态码不是成功状态，或响应体无法解码为文本时返回错误。
pub async fn fetch_subscription(url: &str) -> Result<FetchedSubscription> {
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .with_context(|| format!("fetch proxy subscription `{url}`"))?
        .error_for_status()
        .with_context(|| format!("proxy subscription returned non-success status `{url}`"))?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = response
        .text()
        .await
        .with_context(|| format!("read proxy subscription body `{url}`"))?;

    tracing::debug!(
        url,
        content_type = content_type.as_deref().unwrap_or(""),
        bytes = body.len(),
        "fetched proxy subscription"
    );

    Ok(FetchedSubscription { body, content_type })
}

/// 获取订阅 URL 并解析出所有受支持的代理节点。
///
/// 解析器会自动识别 Clash YAML、明文 URI 列表和 base64 编码 URI 列表，
/// 同时把 HTTP `content-type` 响应头作为辅助判断。
///
/// # Errors
///
/// 当订阅获取失败，或响应中没有任何可用节点时返回错误。
pub async fn fetch_and_parse(url: &str) -> Result<Vec<ProxyNode>> {
    let fetched = fetch_subscription(url).await?;
    parse_subscription(&fetched.body, fetched.content_type.as_deref())
}
