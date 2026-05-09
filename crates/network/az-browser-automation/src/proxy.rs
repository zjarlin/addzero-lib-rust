//! Proxy configuration for isolated browser sessions.

use crate::{BrowserAutomationError, BrowserAutomationResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Proxy protocol supported by Chrome launch arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    /// HTTP proxy.
    Http,
    /// SOCKS5 proxy.
    Socks5,
}

impl ProxyType {
    /// Returns the URI scheme used by Chrome's `--proxy-server` argument.
    #[must_use]
    pub const fn as_scheme(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Socks5 => "socks5",
        }
    }
}

impl FromStr for ProxyType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "http" | "https" => Ok(Self::Http),
            "socks5" | "socks" => Ok(Self::Socks5),
            other => Err(format!("unsupported proxy scheme `{other}`")),
        }
    }
}

/// Proxy endpoint and optional credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy protocol.
    pub proxy_type: ProxyType,
    /// Proxy host or IP address.
    pub host: String,
    /// Proxy port.
    pub port: u16,
    /// Optional username parsed from the URL user-info section.
    pub username: Option<String>,
    /// Optional password parsed from the URL user-info section.
    pub password: Option<String>,
}

impl ProxyConfig {
    /// Creates a proxy configuration from explicit parts.
    #[must_use]
    pub fn new(proxy_type: ProxyType, host: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            host: host.into(),
            port,
            username: None,
            password: None,
        }
    }

    /// Adds proxy credentials to this configuration.
    #[must_use]
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Parses a proxy URL such as `socks5://user:pass@127.0.0.1:1080`.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::InvalidProxyUrl`] if the URL is
    /// missing a scheme, host, or valid port, or if the scheme is unsupported.
    pub fn from_url(url: &str) -> BrowserAutomationResult<Self> {
        let original = url.trim();
        let (scheme, rest) = original
            .split_once("://")
            .ok_or_else(|| invalid_proxy_url(original, "missing scheme separator `://`"))?;
        let proxy_type = scheme
            .parse::<ProxyType>()
            .map_err(|message| invalid_proxy_url(original, message))?;
        let authority = rest.split('/').next().unwrap_or_default();

        if authority.is_empty() {
            return Err(invalid_proxy_url(original, "missing proxy authority"));
        }

        let (userinfo, host_port) = authority
            .rsplit_once('@')
            .map_or((None, authority), |(userinfo, host_port)| {
                (Some(userinfo), host_port)
            });
        let (host, port) = split_host_port(host_port, original)?;
        let (username, password) = parse_userinfo(userinfo);

        Ok(Self {
            proxy_type,
            host,
            port,
            username,
            password,
        })
    }

    /// Loads a proxy pool file with one proxy URL per line.
    ///
    /// Empty lines and `#` comments are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError::ProxyPoolIo`] when the file cannot be
    /// read, or [`BrowserAutomationError::InvalidProxyUrl`] for an invalid
    /// non-comment line.
    pub fn load_pool(path: impl AsRef<Path>) -> BrowserAutomationResult<Vec<Self>> {
        let path = path.as_ref();
        let contents =
            fs::read_to_string(path).map_err(|source| BrowserAutomationError::ProxyPoolIo {
                path: path.to_path_buf(),
                source,
            })?;
        contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(Self::from_url)
            .collect()
    }

    /// Returns Chrome's proxy launch argument for this proxy.
    ///
    /// Chrome does not accept embedded credentials in `--proxy-server`; callers
    /// that need authenticated proxies must provide credentials through a
    /// browser-supported authentication flow.
    #[must_use]
    pub fn chrome_arg(&self) -> String {
        format!(
            "--proxy-server={}://{}:{}",
            self.proxy_type.as_scheme(),
            self.host,
            self.port
        )
    }
}

fn split_host_port(host_port: &str, original: &str) -> BrowserAutomationResult<(String, u16)> {
    if let Some(after_bracket) = host_port.strip_prefix('[') {
        let (host, tail) = after_bracket
            .split_once(']')
            .ok_or_else(|| invalid_proxy_url(original, "unterminated IPv6 host"))?;
        let port = tail
            .strip_prefix(':')
            .ok_or_else(|| invalid_proxy_url(original, "missing IPv6 proxy port"))?;
        return Ok((host.to_owned(), parse_port(port, original)?));
    }

    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| invalid_proxy_url(original, "missing proxy port"))?;
    if host.trim().is_empty() {
        return Err(invalid_proxy_url(original, "missing proxy host"));
    }
    Ok((host.to_owned(), parse_port(port, original)?))
}

fn parse_port(port: &str, original: &str) -> BrowserAutomationResult<u16> {
    port.parse::<u16>()
        .map_err(|_| invalid_proxy_url(original, "invalid proxy port"))
}

fn parse_userinfo(userinfo: Option<&str>) -> (Option<String>, Option<String>) {
    userinfo.map_or((None, None), |value| {
        let (username, password) = value.split_once(':').unwrap_or((value, ""));
        (non_empty_string(username), non_empty_string(password))
    })
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn invalid_proxy_url(url: &str, message: impl Into<String>) -> BrowserAutomationError {
    BrowserAutomationError::InvalidProxyUrl {
        url: url.to_owned(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_url_should_parse_socks5_with_credentials() -> BrowserAutomationResult<()> {
        let proxy = ProxyConfig::from_url("socks5://user:pass@example.test:1080")?;

        assert_eq!(
            proxy,
            ProxyConfig {
                proxy_type: ProxyType::Socks5,
                host: "example.test".to_owned(),
                port: 1080,
                username: Some("user".to_owned()),
                password: Some("pass".to_owned()),
            }
        );
        Ok(())
    }

    #[test]
    fn chrome_arg_should_exclude_credentials() -> BrowserAutomationResult<()> {
        let proxy = ProxyConfig::from_url("http://user:pass@127.0.0.1:8080")?;

        assert_eq!(proxy.chrome_arg(), "--proxy-server=http://127.0.0.1:8080");
        Ok(())
    }

    #[test]
    fn from_url_should_reject_missing_port() {
        let error = ProxyConfig::from_url("socks5://example.test").err();

        assert!(matches!(
            error,
            Some(BrowserAutomationError::InvalidProxyUrl { .. })
        ));
    }
}
