use crate::{CodexAuthSupportError, CodexAuthSupportResult, DuckMailConfig};
use az_derive_aliases::{apply, plain_clone_debug};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;

#[apply(plain_clone_debug)]
pub(crate) struct HttpClient {
    base_url: Url,
    client: Client,
}

impl HttpClient {
    pub(crate) fn new(config: &DuckMailConfig) -> CodexAuthSupportResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| CodexAuthSupportError::InvalidBaseUrl(config.base_url.clone()))?;

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
            client: builder.build()?,
        })
    }

    pub(crate) fn get(&self, path: &str) -> CodexAuthSupportResult<RequestBuilder> {
        Ok(self.client.get(self.join_url(path)?))
    }

    pub(crate) fn post(&self, path: &str) -> CodexAuthSupportResult<RequestBuilder> {
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

    pub(crate) fn read_json<T: DeserializeOwned>(response: Response) -> CodexAuthSupportResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    pub(crate) fn ensure_success(response: Response) -> CodexAuthSupportResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(CodexAuthSupportError::Transport(error)),
        };

        Err(CodexAuthSupportError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> CodexAuthSupportResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| CodexAuthSupportError::InvalidPath(path.to_owned()))
    }
}
