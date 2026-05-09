use crate::http::HttpClient;
use crate::{CodexAuthFile, CodexAuthSupportResult, CpaUploadConfig};
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use std::path::Path;

/// Blocking client for CLIProxyAPI-compatible auth-file upload endpoints.
#[derive(Debug, Clone)]
pub struct CpaClient {
    config: CpaUploadConfig,
    client: Client,
}

impl CpaClient {
    /// Creates a management upload client from validated configuration.
    pub fn new(config: CpaUploadConfig) -> CodexAuthSupportResult<Self> {
        let config = config.build()?;
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            config,
            client: builder.build()?,
        })
    }

    /// Uploads an existing auth JSON file as multipart field `file`.
    pub fn upload_file(&self, path: impl AsRef<Path>) -> CodexAuthSupportResult<()> {
        let form = Form::new().file("file", path.as_ref())?;
        let response = self.request()?.multipart(form).send()?;
        HttpClient::ensure_success(response)?;
        Ok(())
    }

    /// Serializes and uploads an auth file without first writing it to disk.
    pub fn upload_auth_file(
        &self,
        auth_file: &CodexAuthFile,
        file_name: impl Into<String>,
    ) -> CodexAuthSupportResult<()> {
        let part = Part::bytes(serde_json::to_vec(auth_file)?)
            .file_name(file_name.into())
            .mime_str("application/json")?;
        let form = Form::new().part("file", part);
        let response = self.request()?.multipart(form).send()?;
        HttpClient::ensure_success(response)?;
        Ok(())
    }

    fn request(&self) -> CodexAuthSupportResult<reqwest::blocking::RequestBuilder> {
        let url = Url::parse(&self.config.upload_url).map_err(|_| {
            crate::CodexAuthSupportError::InvalidBaseUrl(self.config.upload_url.clone())
        })?;
        let builder = self.client.post(url);
        Ok(match self.config.bearer_token.as_deref() {
            Some(token) if !token.trim().is_empty() => builder.bearer_auth(token),
            _ => builder,
        })
    }
}
