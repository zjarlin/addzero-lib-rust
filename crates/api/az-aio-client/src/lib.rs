#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use az_config_center_contract::{
    DESKTOP_SESSION_TOKEN_HEADER, DesktopBackendStatus, ShellComponent, ShellComponentBuildRequest,
    ShellComponentBuildResult, ShellComponentConfigUpdate, ShellComponentPatch,
    ShellComponentRegistry, ShellComponentRemove, ShellComponentUpsert,
};
use az_derive_aliases::{apply, plain_clone_debug};
use reqwest::Method;
use reqwest::blocking::{Client, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

pub type AioClientResult<T> = Result<T, AioClientError>;

#[derive(Debug, Error)]
pub enum AioClientError {
    #[error("request to {url} failed: {source}")]
    Transport {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("request to {url} returned {status}: {body}")]
    Http {
        url: String,
        status: reqwest::StatusCode,
        body: String,
    },
}

#[apply(plain_clone_debug)]
pub struct AioClient {
    base_url: String,
    desktop_token: Option<String>,
    http: Client,
}

impl AioClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: normalize_base_url(base_url.into()),
            desktop_token: None,
            http: Client::new(),
        }
    }

    pub fn with_desktop_token(
        base_url: impl Into<String>,
        desktop_token: impl Into<String>,
    ) -> Self {
        Self {
            base_url: normalize_base_url(base_url.into()),
            desktop_token: Some(desktop_token.into()),
            http: Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn desktop_status(&self) -> AioClientResult<DesktopBackendStatus> {
        self.request(Method::GET, "/api/desktop/status", None::<&()>)
    }

    pub fn list_shell_components(&self) -> AioClientResult<ShellComponentRegistry> {
        self.request(Method::GET, "/api/shell-components", None::<&()>)
    }

    pub fn get_shell_component(&self, name: &str) -> AioClientResult<Option<ShellComponent>> {
        self.request(
            Method::GET,
            &format!("/api/shell-components/{}", urlencoding::encode(name)),
            None::<&()>,
        )
    }

    pub fn upsert_shell_component(
        &self,
        input: &ShellComponentUpsert,
    ) -> AioClientResult<ShellComponent> {
        self.request(Method::POST, "/api/shell-components/upsert", Some(input))
    }

    pub fn patch_shell_component(
        &self,
        input: &ShellComponentPatch,
    ) -> AioClientResult<ShellComponent> {
        self.request(Method::POST, "/api/shell-components/patch", Some(input))
    }

    pub fn remove_shell_component(
        &self,
        input: &ShellComponentRemove,
    ) -> AioClientResult<ShellComponent> {
        self.request(Method::POST, "/api/shell-components/remove", Some(input))
    }

    pub fn save_shell_component_config(
        &self,
        input: &ShellComponentConfigUpdate,
    ) -> AioClientResult<ShellComponentRegistry> {
        self.request(Method::POST, "/api/shell-components/config", Some(input))
    }

    pub fn build_shell_components(
        &self,
        input: &ShellComponentBuildRequest,
    ) -> AioClientResult<ShellComponentBuildResult> {
        self.request(Method::POST, "/api/shell-components/build", Some(input))
    }

    fn request<T, B>(&self, method: Method, path: &str, body: Option<B>) -> AioClientResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.http.request(method, &url);
        if let Some(token) = &self.desktop_token {
            request = request.header(DESKTOP_SESSION_TOKEN_HEADER, token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().map_err(|source| AioClientError::Transport {
            url: url.clone(),
            source,
        })?;
        decode_json(url, response)
    }
}

fn normalize_base_url(base_url: String) -> String {
    base_url.trim_end_matches('/').to_string()
}

fn decode_json<T>(url: String, response: Response) -> AioClientResult<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(AioClientError::Http { url, status, body });
    }
    response
        .json()
        .map_err(|source| AioClientError::Transport { url, source })
}
