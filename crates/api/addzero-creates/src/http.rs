use crate::util::trim_non_blank;
use crate::{ApiConfig, CreatesError, CreatesResult};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    pub(crate) fn new(config: ApiConfig) -> CreatesResult<Self> {
        config.validate().map_err(CreatesError::from)?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| CreatesError::InvalidBaseUrl(config.base_url.clone()))?;
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
            client: builder.build()?,
        })
    }

    pub(crate) fn get(&self, path: &str) -> CreatesResult<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    pub(crate) fn get_url(&self, url: Url) -> RequestBuilder {
        self.client.get(url)
    }

    pub(crate) fn post(&self, path: &str) -> CreatesResult<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
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

    pub(crate) fn with_headers(
        builder: RequestBuilder,
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<RequestBuilder> {
        if headers.is_empty() {
            return Ok(builder);
        }
        Ok(builder.headers(build_header_map(headers)?))
    }

    pub(crate) fn read_json<T: DeserializeOwned>(response: Response) -> CreatesResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    pub(crate) fn read_bytes(response: Response) -> CreatesResult<Vec<u8>> {
        let response = Self::ensure_success(response)?;
        Ok(response.bytes()?.to_vec())
    }

    pub(crate) fn build_url(&self, path: &str, query: &[(&str, String)]) -> CreatesResult<Url> {
        let mut url = self.join_url(path)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        Ok(url)
    }

    fn ensure_success(response: Response) -> CreatesResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(CreatesError::Transport(error)),
        };

        Err(CreatesError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> CreatesResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| CreatesError::InvalidPath(path.to_owned()))
    }
}

fn build_header_map(headers: &BTreeMap<String, String>) -> CreatesResult<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|source| {
            CreatesError::InvalidHeaderName {
                name: name.clone(),
                source,
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|source| CreatesError::InvalidHeaderValue {
                name: name.clone(),
                source,
            })?;
        header_map.insert(header_name, header_value);
    }
    Ok(header_map)
}
