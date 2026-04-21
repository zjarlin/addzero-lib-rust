use crate::util::trim_non_blank;
use crate::{ApiConfig, CreatesError, CreatesResult};
use reqwest::blocking::{Client, Response};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue,
};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct HttpApiClient {
    base_url: Url,
    client: Client,
}

impl HttpApiClient {
    pub(crate) fn new(config: ApiConfig) -> CreatesResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| CreatesError::InvalidBaseUrl(config.base_url))?;
        let default_headers = build_default_headers(&config.default_headers)?;

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

    pub(crate) fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        bearer_token: Option<&str>,
    ) -> CreatesResult<T> {
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        self.get_json_with_headers(path, query, &headers)
    }

    pub(crate) fn get_json_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let url = self.build_url(path, query)?;
        self.get_json_url_with_headers(url, headers)
    }

    pub(crate) fn get_json_url_with_headers<T: DeserializeOwned>(
        &self,
        url: Url,
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let builder = self.client.request(Method::GET, url.clone());
        let builder = apply_headers(builder, headers)?;
        let response = builder.send()?;
        self.read_json(url, response)
    }

    pub(crate) fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        bearer_token: Option<&str>,
    ) -> CreatesResult<T> {
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        self.post_json_with_headers(path, body, &headers)
    }

    pub(crate) fn post_json_with_headers<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: &BTreeMap<String, String>,
    ) -> CreatesResult<T> {
        let url = self.build_url(path, &[])?;
        let body_bytes = serde_json::to_vec(body)?;
        let builder = self
            .client
            .request(Method::POST, url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(body_bytes);
        let builder = apply_headers(builder, headers)?;
        let response = builder.send()?;
        self.read_json(url, response)
    }

    pub(crate) fn get_bytes(
        &self,
        path: &str,
        query: &[(&str, String)],
        bearer_token: Option<&str>,
    ) -> CreatesResult<Vec<u8>> {
        let mut headers = BTreeMap::new();
        if let Some(token) = trim_non_blank(bearer_token) {
            headers.insert(AUTHORIZATION.as_str().to_owned(), format!("Bearer {token}"));
        }
        let url = self.build_url(path, query)?;
        let builder = self.client.request(Method::GET, url.clone());
        let builder = apply_headers(builder, &headers)?;
        let response = builder.send()?;
        let response = self.ensure_success(&url, response)?;
        Ok(response.bytes()?.to_vec())
    }

    pub(crate) fn build_url(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> CreatesResult<Url> {
        let mut url = self.join_url(path)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        Ok(url)
    }

    fn read_json<T: DeserializeOwned>(&self, url: Url, response: Response) -> CreatesResult<T> {
        let response = self.ensure_success(&url, response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    fn ensure_success(&self, url: &Url, response: Response) -> CreatesResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(CreatesError::Transport(error)),
        };

        Err(CreatesError::HttpStatus {
            url: url.to_string(),
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

fn build_default_headers(headers: &BTreeMap<String, String>) -> CreatesResult<HeaderMap> {
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

fn apply_headers(
    mut builder: reqwest::blocking::RequestBuilder,
    headers: &BTreeMap<String, String>,
) -> CreatesResult<reqwest::blocking::RequestBuilder> {
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
        builder = builder.header(header_name, header_value);
    }
    Ok(builder)
}
