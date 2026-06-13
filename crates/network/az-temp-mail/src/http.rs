use crate::util::trim_non_blank;
use crate::config::ApiConfig;
use anyhow::{Context, bail};
use az_derive_aliases::{apply, plain_clone_debug};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

#[apply(plain_clone_debug)]
pub(crate) struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    pub(crate) fn new(config: ApiConfig) -> anyhow::Result<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .with_context(|| format!("invalid base url `{}`", config.base_url))?;
        let default_headers = build_header_map(&config.default_headers)?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .default_headers(default_headers);

        if let Some(user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            client: builder
                .build()
                .context("failed to build temp-mail HTTP client")?,
        })
    }

    pub(crate) fn get(&self, path: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    pub(crate) fn get_with_query(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> anyhow::Result<RequestBuilder> {
        let mut url = self.join_url(path)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        Ok(self.client.get(url))
    }

    pub(crate) fn post(&self, path: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    pub(crate) fn delete(&self, path: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.client.delete(self.join_url(path)?))
    }

    pub(crate) fn with_bearer_auth(
        builder: RequestBuilder,
        bearer_token: Option<&str>,
    ) -> RequestBuilder {
        match trim_non_blank(bearer_token) {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    pub(crate) fn read_json<T: DeserializeOwned>(response: Response) -> anyhow::Result<T> {
        let response = Self::ensure_success(response)?;
        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read response body from `{url}`"))?;
        serde_json::from_slice(bytes.as_ref())
            .with_context(|| format!("failed to parse JSON response from `{url}`"))
    }

    fn ensure_success(response: Response) -> anyhow::Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read error response body from `{url}`"))?;
        bail!(
            "request to `{url}` returned HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(bytes.as_ref())
        )
    }

    fn join_url(&self, path: &str) -> anyhow::Result<Url> {
        self.base_url
            .join(path)
            .with_context(|| format!("invalid request path `{path}`"))
    }
}

fn build_header_map(headers: &BTreeMap<String, String>) -> anyhow::Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .with_context(|| format!("invalid header name `{name}`"))?;
        let header_value = HeaderValue::from_str(value)
            .with_context(|| format!("invalid header value for `{name}`"))?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}
