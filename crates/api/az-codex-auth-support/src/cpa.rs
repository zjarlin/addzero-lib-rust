use crate::http::HttpClient;
use crate::{auth_file::CodexAuthFile, config::CpaUploadConfig};
use anyhow::Context;
use az_derive_aliases::{apply, plain_clone_debug};
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::blocking::multipart::{Form, Part};
use std::path::Path;

/// CLIProxyAPI 兼容认证文件上传端点的阻塞客户端。
#[apply(plain_clone_debug)]
pub struct CpaClient {
    config: CpaUploadConfig,
    client: Client,
}

impl CpaClient {
    /// 根据已校验配置创建管理端上传客户端。
    pub fn new(config: CpaUploadConfig) -> anyhow::Result<Self> {
        let config = config.build()?;
        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            config,
            client: builder
                .build()
                .context("failed to build CPA upload HTTP client")?,
        })
    }

    /// 将已有认证 JSON 文件作为 multipart 字段 `file` 上传。
    pub fn upload_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let form = Form::new()
            .file("file", path.as_ref())
            .with_context(|| format!("failed to attach file `{}`", path.as_ref().display()))?;
        let response = self
            .request()?
            .multipart(form)
            .send()
            .with_context(|| format!("failed to upload file `{}`", path.as_ref().display()))?;
        HttpClient::ensure_success(response)?;
        Ok(())
    }

    /// 不落盘，直接序列化并上传认证文件。
    pub fn upload_auth_file(
        &self,
        auth_file: &CodexAuthFile,
        file_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let file_name = file_name.into();
        let part = Part::bytes(
            serde_json::to_vec(auth_file)
                .context("failed to serialize Codex auth file for upload")?,
        )
        .file_name(file_name.clone())
        .mime_str("application/json")?;
        let form = Form::new().part("file", part);
        let response = self
            .request()?
            .multipart(form)
            .send()
            .with_context(|| format!("failed to upload auth file `{file_name}`"))?;
        HttpClient::ensure_success(response)?;
        Ok(())
    }

    fn request(&self) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
        let url = Url::parse(&self.config.upload_url)
            .with_context(|| format!("invalid base url `{}`", self.config.upload_url))?;
        let builder = self.client.post(url);
        Ok(match self.config.bearer_token.as_deref() {
            Some(token) if !token.trim().is_empty() => builder.bearer_auth(token),
            _ => builder,
        })
    }
}
