use crate::{KiroAuthSupportError, KiroAuthSupportResult, KiroOidcConfig};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
    base_url: Url,
    client: Client,
}

impl HttpClient {
    pub(crate) fn new(config: &KiroOidcConfig) -> KiroAuthSupportResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| KiroAuthSupportError::InvalidBaseUrl(config.base_url.clone()))?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            client: builder.build()?,
        })
    }

    pub(crate) fn post(&self, path: &str) -> KiroAuthSupportResult<RequestBuilder> {
        Ok(self.client.post(self.join_url(path)?))
    }

    pub(crate) fn read_json<T: DeserializeOwned>(response: Response) -> KiroAuthSupportResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    pub(crate) fn ensure_success(response: Response) -> KiroAuthSupportResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(KiroAuthSupportError::Transport(error)),
        };

        Err(KiroAuthSupportError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> KiroAuthSupportResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| KiroAuthSupportError::InvalidPath(path.to_owned()))
    }
}
