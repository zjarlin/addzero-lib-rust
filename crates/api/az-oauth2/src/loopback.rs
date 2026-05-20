use crate::{OAuth2Error, OAuth2Result, PkcePair};
use reqwest::Url;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

/// Active loopback authorization-code session.
#[derive(Debug)]
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
    ) -> OAuth2Result<Self> {
        listener.set_nonblocking(true)?;
        Ok(Self {
            authorization_url,
            redirect_uri,
            state,
            pkce,
            listener,
        })
    }

    /// Waits for one OAuth redirect callback and validates the `state` parameter.
    pub fn wait_for_callback(self, timeout: Duration) -> OAuth2Result<OAuth2AuthorizationCallback> {
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
                    stream.write_all(response.as_bytes())?;

                    if callback.state.as_deref() != Some(self.state.as_str()) {
                        return Err(OAuth2Error::StateMismatch {
                            expected: self.state,
                            actual: callback.state.unwrap_or_default(),
                        });
                    }
                    if let Some(error) = callback.error.as_deref() {
                        return Err(OAuth2Error::ProviderError {
                            error: error.to_owned(),
                            description: callback.error_description.unwrap_or_default(),
                        });
                    }
                    if callback.code.trim().is_empty() {
                        return Err(OAuth2Error::InvalidCallback(
                            "missing authorization code".to_owned(),
                        ));
                    }
                    return Ok(callback);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(OAuth2Error::InvalidCallback(
                            "timed out waiting for loopback callback".to_owned(),
                        ));
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => return Err(OAuth2Error::Io(error)),
            }
        }
    }
}

/// Parsed OAuth loopback callback.
#[derive(Debug, Clone, PartialEq, Eq)]
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

fn read_request(stream: &mut std::net::TcpStream) -> OAuth2Result<String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let read = stream.read(&mut chunk)?;
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
) -> OAuth2Result<OAuth2AuthorizationCallback> {
    let request_target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| OAuth2Error::InvalidCallback("missing HTTP request target".to_owned()))?;
    let redirect = Url::parse(redirect_uri)
        .map_err(|_| OAuth2Error::InvalidBaseUrl(redirect_uri.to_owned()))?;
    let callback_url = format!(
        "{}://{}{}",
        redirect.scheme(),
        redirect
            .host_str()
            .ok_or_else(|| OAuth2Error::InvalidCallback("redirect_uri has no host".to_owned()))?,
        request_target
    );
    let parsed = Url::parse(&callback_url)
        .map_err(|_| OAuth2Error::InvalidCallback(request_target.to_owned()))?;

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
