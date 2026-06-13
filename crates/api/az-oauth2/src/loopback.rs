use crate::pkce::PkcePair;
use anyhow::{Context, anyhow, bail};
use az_derive_aliases::{apply, plain_debug, plain_eq};
use reqwest::Url;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

/// Active loopback authorization-code session.
#[apply(plain_debug)]
pub struct LoopbackAuthorizationSession {
    /// URL the user should open in the system browser.
    pub authorization_url: String,
    /// Redirect URI bound by this session.
    pub redirect_uri: String,
    /// Generated OAuth state expected in the callback.
    pub state: String,
    /// PKCE material required for token exchange.
    pub pkce: PkcePair,
    listener: TcpListener,
}

impl LoopbackAuthorizationSession {
    pub(crate) fn new(
        authorization_url: String,
        redirect_uri: String,
        state: String,
        pkce: PkcePair,
        listener: TcpListener,
    ) -> anyhow::Result<Self> {
        listener
            .set_nonblocking(true)
            .context("failed to configure OAuth2 loopback listener as nonblocking")?;
        Ok(Self {
            authorization_url,
            redirect_uri,
            state,
            pkce,
            listener,
        })
    }

    /// Waits for one OAuth redirect callback and validates the `state` parameter.
    pub fn wait_for_callback(self, timeout: Duration) -> anyhow::Result<OAuth2AuthorizationCallback> {
        let deadline = Instant::now() + timeout;
        loop {
            match self.listener.accept() {
                Ok((mut stream, _)) => {
                    let request = read_request(&mut stream)?;
                    let callback = parse_callback_request(&request, &self.redirect_uri)?;
                    let response = if callback.error.is_some() {
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: 26\r\nConnection: close\r\n\r\nOAuth authorization failed"
                    } else {
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 36\r\nConnection: close\r\n\r\nOAuth authorization received. Return."
                    };
                    stream
                        .write_all(response.as_bytes())
                        .context("failed to write OAuth2 loopback response")?;

                    if callback.state.as_deref() != Some(self.state.as_str()) {
                        let actual = callback.state.unwrap_or_default();
                        bail!(
                            "oauth state mismatch: expected `{}`, got `{actual}`",
                            self.state
                        );
                    }
                    if let Some(error) = callback.error.as_deref() {
                        let description = callback.error_description.unwrap_or_default();
                        bail!("oauth provider error `{error}`: {description}");
                    }
                    if callback.code.trim().is_empty() {
                        bail!("invalid authorization callback: missing authorization code");
                    }
                    return Ok(callback);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        bail!("invalid authorization callback: timed out waiting for loopback callback");
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => {
                    let error =
                        anyhow::Error::from(error).context("failed to accept OAuth2 loopback callback");

                    return Err(error);
                }
            }
        }
    }
}

/// Parsed OAuth loopback callback.
#[apply(plain_eq)]
pub struct OAuth2AuthorizationCallback {
    /// Authorization code returned by the provider.
    pub code: String,
    /// Callback state returned by the provider.
    pub state: Option<String>,
    /// Redirect URI used by this callback.
    pub redirect_uri: String,
    /// Optional OAuth error code.
    pub error: Option<String>,
    /// Optional OAuth error description.
    pub error_description: Option<String>,
}

fn read_request(stream: &mut std::net::TcpStream) -> anyhow::Result<String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream
            .read(&mut chunk)
            .context("failed to read OAuth2 loopback request")?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn parse_callback_request(
    request: &str,
    redirect_uri: &str,
) -> anyhow::Result<OAuth2AuthorizationCallback> {
    let request_target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow!("invalid authorization callback: missing HTTP request target"))?;
    let redirect = Url::parse(redirect_uri)
        .with_context(|| format!("invalid base url `{redirect_uri}`"))?;
    let callback_url = format!(
        "{}://{}{}",
        redirect.scheme(),
        redirect
            .host_str()
            .ok_or_else(|| anyhow!("invalid authorization callback: redirect_uri has no host"))?,
        request_target
    );
    let parsed = Url::parse(&callback_url)
        .with_context(|| format!("invalid authorization callback: {request_target}"))?;

    let params = parsed
        .query_pairs()
        .into_owned()
        .collect::<BTreeMap<_, _>>();
    Ok(OAuth2AuthorizationCallback {
        code: params.get("code").cloned().unwrap_or_default(),
        state: params.get("state").cloned(),
        redirect_uri: redirect_uri.to_owned(),
        error: params.get("error").cloned(),
        error_description: params.get("error_description").cloned(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_callback_request;

    #[test]
    fn callback_parser_reads_code_and_state() {
        let callback = parse_callback_request(
            "GET /oauth/callback?code=code-1&state=state-1 HTTP/1.1\r\nHost: localhost\r\n\r\n",
            "http://127.0.0.1:9000/oauth/callback",
        )
        .expect("callback");

        assert_eq!(callback.code, "code-1");
        assert_eq!(callback.state.as_deref(), Some("state-1"));
    }
}
