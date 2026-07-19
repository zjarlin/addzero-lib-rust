use crate::config::DuckMailConfig;
use anyhow::{Context, bail};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;

#[derive(Clone, Debug)]
pub(crate) struct HttpClient {
    base_url: Url,
    client: Client,
}

impl HttpClient {
    pub(crate) fn new(config: &DuckMailConfig) -> anyhow::Result<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .with_context(|| format!("invalid base url `{}`", config.base_url))?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .default_headers(default_headers);

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            client: builder
                .build()
                .context("failed to build DuckMail HTTP client")?,
        })
    }

    pub(crate) fn get(&self, path: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    pub(crate) fn post(&self, path: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    pub(crate) fn with_bearer_auth(
        builder: RequestBuilder,
        bearer_token: Option<&str>,
    ) -> RequestBuilder {
        match bearer_token
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
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

    pub(crate) fn ensure_success(response: Response) -> anyhow::Result<Response> {
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
