use crate::config::{AuthorizationCodeOptions, OAuth2Config};
use crate::loopback::LoopbackAuthorizationSession;
use crate::model::{OAuth2DeviceAuthorization, OAuth2DeviceTokenPoll, OAuth2TokenResponse};
use crate::pkce::{PkcePair, generate_pkce_pair, generate_state};
use anyhow::{Context, anyhow, bail};
use az_str::api::ensure_leading_slash;
use reqwest::Url;
use reqwest::blocking::{Client, Response};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::net::TcpListener;
use std::time::Duration;

const AUTHORIZATION_CODE_GRANT_TYPE: &str = "authorization_code";
const REFRESH_TOKEN_GRANT_TYPE: &str = "refresh_token";
const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// Blocking OAuth2 client for authorization-code, refresh-token, and device flows.
#[derive(Clone, Debug)]
pub struct OAuth2Client {
    config: OAuth2Config,
    authorization_url: Url,
    token_url: Url,
    device_authorization_url: Option<Url>,
    client: Client,
}

impl OAuth2Client {
    /// Creates a client from validated config.
    pub fn new(config: OAuth2Config) -> anyhow::Result<Self> {
        config.validate()?;
        let authorization_url = Url::parse(&config.authorization_url)
            .with_context(|| format!("invalid base url `{}`", config.authorization_url))?;
        let token_url = Url::parse(&config.token_url)
            .with_context(|| format!("invalid base url `{}`", config.token_url))?;
        let device_authorization_url = config
            .device_authorization_url
            .as_ref()
            .map(|url| Url::parse(url).with_context(|| format!("invalid base url `{url}`")))
            .transpose()?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);
        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            config,
            authorization_url,
            token_url,
            device_authorization_url,
            client: builder.build().context("failed to build OAuth2 HTTP client")?,
        })
    }

    /// Builds an authorization URL and generated state/PKCE material.
    pub fn build_authorization_url(
        &self,
        options: AuthorizationCodeOptions,
    ) -> anyhow::Result<AuthorizationRequest> {
        let redirect_uri = options
            .redirect_uri
            .clone()
            .or_else(|| self.config.redirect_uri.clone())
            .ok_or_else(|| anyhow!("invalid config: redirect_uri is required"))?;
        let state = options.state.clone().map_or_else(generate_state, Ok)?;
        let pkce = options.pkce.clone().map_or_else(generate_pkce_pair, Ok)?;
        let scopes = effective_scopes(&options.scopes, &self.config.scopes);

        let mut url = self.authorization_url.clone();
        {
            let mut pairs = url.query_pairs_mut();
            pairs
                .append_pair("response_type", "code")
                .append_pair("client_id", &self.config.client_id)
                .append_pair("redirect_uri", &redirect_uri)
                .append_pair("state", &state)
                .append_pair("code_challenge", &pkce.code_challenge)
                .append_pair("code_challenge_method", &pkce.code_challenge_method);
            if !scopes.is_empty() {
                pairs.append_pair("scope", &scopes.join(" "));
            }
            if let Some(value) = trimmed(options.login_hint.as_deref()) {
                pairs.append_pair("login_hint", value);
            }
            if let Some(value) = trimmed(options.access_type.as_deref()) {
                pairs.append_pair("access_type", value);
            }
            if let Some(value) = trimmed(options.prompt.as_deref()) {
                pairs.append_pair("prompt", value);
            }
            for (name, value) in options.extra_params {
                if !name.trim().is_empty() {
                    pairs.append_pair(&name, &value);
                }
            }
        }

        Ok(AuthorizationRequest {
            authorization_url: url.to_string(),
            redirect_uri,
            state,
            pkce,
        })
    }

    /// Starts a local loopback listener and builds the matching authorization URL.
    pub fn begin_loopback_authorization(
        &self,
        mut options: AuthorizationCodeOptions,
    ) -> anyhow::Result<LoopbackAuthorizationSession> {
        let listener = TcpListener::bind(&options.loopback_bind_addr).with_context(|| {
            format!("failed to bind OAuth2 loopback listener `{}`", options.loopback_bind_addr)
        })?;
        let local_addr = listener
            .local_addr()
            .context("failed to read OAuth2 loopback listener address")?;
        let path = ensure_leading_slash(&options.loopback_path);
        let redirect_uri = format!("http://127.0.0.1:{}{path}", local_addr.port());
        options.redirect_uri = Some(redirect_uri.clone());

        let request = self.build_authorization_url(options)?;
        LoopbackAuthorizationSession::new(
            request.authorization_url,
            redirect_uri,
            request.state,
            request.pkce,
            listener,
        )
    }

    /// Exchanges an authorization code for tokens.
    pub fn exchange_authorization_code(
        &self,
        code: impl AsRef<str>,
        redirect_uri: impl AsRef<str>,
        pkce: Option<&PkcePair>,
    ) -> anyhow::Result<OAuth2TokenResponse> {
        let mut form = self.base_token_form(AUTHORIZATION_CODE_GRANT_TYPE);
        form.insert("code".to_owned(), code.as_ref().to_owned());
        form.insert("redirect_uri".to_owned(), redirect_uri.as_ref().to_owned());
        if let Some(pkce) = pkce {
            form.insert("code_verifier".to_owned(), pkce.code_verifier.clone());
        }

        self.send_token_form(&form)?.into_success()
    }

    /// Exchanges a refresh token for a new access token.
    pub fn refresh_access_token(
        &self,
        refresh_token: impl AsRef<str>,
    ) -> anyhow::Result<OAuth2TokenResponse> {
        let mut form = self.base_token_form(REFRESH_TOKEN_GRANT_TYPE);
        form.insert(
            "refresh_token".to_owned(),
            refresh_token.as_ref().to_owned(),
        );

        self.send_token_form(&form)?.into_success()
    }

    /// Starts an OAuth2 device authorization flow.
    pub fn begin_device_authorization(&self) -> anyhow::Result<OAuth2DeviceAuthorization> {
        let url = self.device_authorization_url.as_ref().ok_or_else(|| {
            anyhow!("invalid config: device_authorization_url is required for device flow")
        })?;
        let mut form = BTreeMap::new();
        form.insert("client_id".to_owned(), self.config.client_id.clone());
        if !self.config.scopes.is_empty() {
            form.insert("scope".to_owned(), self.config.scopes.join(" "));
        }

        let response = self
            .client
            .post(url.clone())
            .form(&form)
            .send()
            .with_context(|| format!("failed to send device authorization request to `{url}`"))?;
        Self::read_json(response)
    }

    /// Polls the token endpoint once for a device-flow session.
    pub fn poll_device_token_once(
        &self,
        authorization: &OAuth2DeviceAuthorization,
        current_interval_secs: u64,
    ) -> anyhow::Result<OAuth2DeviceTokenPoll> {
        let mut form = self.base_token_form(DEVICE_CODE_GRANT_TYPE);
        form.insert("device_code".to_owned(), authorization.device_code.clone());

        let token = self.send_token_form(&form)?;
        if token.is_success() {
            return Ok(OAuth2DeviceTokenPoll::Success(token));
        }

        let message = token.error_description.clone().unwrap_or_default();
        match token.error.as_deref() {
            Some("authorization_pending") => Ok(OAuth2DeviceTokenPoll::Pending),
            Some("slow_down") => Ok(OAuth2DeviceTokenPoll::SlowDown {
                next_interval_secs: current_interval_secs.saturating_add(5),
            }),
            Some("expired_token") => Ok(OAuth2DeviceTokenPoll::Expired { message }),
            Some("access_denied") => Ok(OAuth2DeviceTokenPoll::AccessDenied { message }),
            _ => Ok(OAuth2DeviceTokenPoll::Error {
                message,
                response: token,
            }),
        }
    }

    /// Polls a device-flow session until success or a terminal error.
    pub fn wait_for_device_token(
        &self,
        authorization: &OAuth2DeviceAuthorization,
        timeout: Duration,
    ) -> anyhow::Result<OAuth2DeviceTokenPoll> {
        let mut interval = authorization.interval.unwrap_or(5).max(1);
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let poll = self.poll_device_token_once(authorization, interval)?;
            match poll {
                OAuth2DeviceTokenPoll::Pending => {}
                OAuth2DeviceTokenPoll::SlowDown { next_interval_secs } => {
                    interval = next_interval_secs.max(1);
                }
                terminal => return Ok(terminal),
            }

            if std::time::Instant::now() >= deadline {
                return Ok(OAuth2DeviceTokenPoll::Expired {
                    message: "device token polling timed out".to_owned(),
                });
            }
            std::thread::sleep(Duration::from_secs(interval));
        }
    }

    fn base_token_form(&self, grant_type: &str) -> BTreeMap<String, String> {
        let mut form = BTreeMap::new();
        form.insert("grant_type".to_owned(), grant_type.to_owned());
        form.insert("client_id".to_owned(), self.config.client_id.clone());
        if let Some(client_secret) = &self.config.client_secret {
            form.insert("client_secret".to_owned(), client_secret.clone());
        }
        form
    }

    fn send_token_form(
        &self,
        form: &BTreeMap<String, String>,
    ) -> anyhow::Result<OAuth2TokenResponse> {
        let response = self
            .client
            .post(self.token_url.clone())
            .form(form)
            .send()
            .with_context(|| format!("failed to send token request to `{}`", self.token_url))?;
        Self::read_token_response(response)
    }

    fn read_json<T: DeserializeOwned>(response: Response) -> anyhow::Result<T> {
        let response = Self::ensure_success(response)?;
        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read response body from `{url}`"))?;
        serde_json::from_slice(bytes.as_ref())
            .with_context(|| format!("failed to parse JSON response from `{url}`"))
    }

    fn read_token_response(response: Response) -> anyhow::Result<OAuth2TokenResponse> {
        let status = response.status();
        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read token response body from `{url}`"))?;
        if status.is_success() {
            return serde_json::from_slice(bytes.as_ref())
                .with_context(|| format!("failed to parse token JSON response from `{url}`"));
        }

        if let Ok(token) = serde_json::from_slice::<OAuth2TokenResponse>(bytes.as_ref())
            && token.error.is_some()
        {
            return Ok(token);
        }

        bail!(
            "request to `{url}` returned HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(bytes.as_ref())
        )
    }

    fn ensure_success(response: Response) -> anyhow::Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let bytes = response
            .bytes()
            .with_context(|| format!("failed to read error response body from `{url}`"))?;
        bail!(
            "request to `{url}` returned HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(bytes.as_ref())
        )
    }
}

/// Built authorization URL and the material needed for token exchange.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest {
    /// URL the user should open in the system browser.
    pub authorization_url: String,
    /// Redirect URI used in the request.
    pub redirect_uri: String,
    /// Generated or caller-provided OAuth state.
    pub state: String,
    /// PKCE material required during token exchange.
    pub pkce: PkcePair,
}

fn effective_scopes(options_scopes: &[String], config_scopes: &[String]) -> Vec<String> {
    let source = if options_scopes.is_empty() {
        config_scopes
    } else {
        options_scopes
    };
    source
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::OAuth2Client;
    use crate::config::{AuthorizationCodeOptions, OAuth2Config};
    use crate::model::{OAuth2DeviceTokenPoll, OAuth2TokenResponse};
    use crate::pkce::PkcePair;
        use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    #[test]
    fn authorization_url_contains_pkce_state_and_google_offline_params() {
        let client = test_client(
            "https://auth.example.com/oauth/authorize",
            "https://token.example.com/token",
        );
        let request = client
            .build_authorization_url(
                AuthorizationCodeOptions::new()
                    .redirect_uri("http://127.0.0.1:9000/oauth/callback")
                    .state("state-1")
                    .pkce(test_pkce())
                    .access_type("offline")
                    .prompt("consent")
                    .login_hint("user@example.com"),
            )
            .expect("authorize url");

        assert!(
            request
                .authorization_url
                .starts_with("https://auth.example.com/oauth/authorize?")
        );
        assert!(request.authorization_url.contains("response_type=code"));
        assert!(request.authorization_url.contains("client_id=client-1"));
        assert!(
            request
                .authorization_url
                .contains("code_challenge=challenge-1")
        );
        assert!(
            request
                .authorization_url
                .contains("code_challenge_method=S256")
        );
        assert!(request.authorization_url.contains("access_type=offline"));
        assert!(request.authorization_url.contains("prompt=consent"));
        assert!(
            request
                .authorization_url
                .contains("login_hint=user%40example.com")
        );
    }

    #[test]
    fn authorization_code_exchange_posts_standard_form() {
        let server = TestServer::spawn(vec![TestResponse::json(
            200,
            r#"{"access_token":"access-1","refresh_token":"refresh-1","token_type":"Bearer","expires_in":3600}"#,
        )]);
        let client = test_client(
            "https://auth.example.com/oauth/authorize",
            &server.url("/token"),
        );

        let token = client
            .exchange_authorization_code(
                "code-1",
                "http://127.0.0.1:9000/oauth/callback",
                Some(&test_pkce()),
            )
            .expect("token");

        assert_eq!(token.require_access_token().expect("access"), "access-1");
        let requests = server.finish();
        assert_eq!(requests[0].path, "/token");
        assert!(requests[0].body.contains("grant_type=authorization_code"));
        assert!(requests[0].body.contains("code=code-1"));
        assert!(requests[0].body.contains("client_id=client-1"));
        assert!(requests[0].body.contains("client_secret=secret-1"));
        assert!(requests[0].body.contains("code_verifier=verifier-1"));
    }

    #[test]
    fn refresh_token_exchange_posts_refresh_grant() {
        let server = TestServer::spawn(vec![TestResponse::json(
            200,
            r#"{"access_token":"access-2","token_type":"Bearer"}"#,
        )]);
        let client = test_client(
            "https://auth.example.com/oauth/authorize",
            &server.url("/token"),
        );

        let token = client
            .refresh_access_token("refresh-1")
            .expect("refresh token");

        assert_eq!(token.require_access_token().expect("access"), "access-2");
        let requests = server.finish();
        assert!(requests[0].body.contains("grant_type=refresh_token"));
        assert!(requests[0].body.contains("refresh_token=refresh-1"));
    }

    #[test]
    fn device_flow_maps_pending_and_success() {
        let server = TestServer::spawn(vec![
            TestResponse::json(
                200,
                r#"{"device_code":"device-1","user_code":"USER-CODE","verification_uri":"https://verify.example.com","interval":1}"#,
            ),
            TestResponse::json(400, r#"{"error":"authorization_pending"}"#),
            TestResponse::json(200, r#"{"access_token":"access-1","token_type":"Bearer"}"#),
        ]);
        let config = OAuth2Config::builder(
            "https://auth.example.com/oauth/authorize",
            server.url("/token"),
            "client-1",
        )
        .device_authorization_url(server.url("/device/code"))
        .scope("scope-a")
        .build()
        .expect("config");
        let client = OAuth2Client::new(config).expect("client");

        let device = client.begin_device_authorization().expect("device auth");
        assert_eq!(device.user_code, "USER-CODE");
        assert!(matches!(
            client.poll_device_token_once(&device, 1).expect("pending"),
            OAuth2DeviceTokenPoll::Pending
        ));
        assert!(matches!(
            client.poll_device_token_once(&device, 1).expect("success"),
            OAuth2DeviceTokenPoll::Success(_)
        ));

        let requests = server.finish();
        assert_eq!(requests[0].path, "/device/code");
        assert!(requests[0].body.contains("scope=scope-a"));
        assert_eq!(requests[1].path, "/token");
        assert!(requests[1].body.contains("device_code=device-1"));
    }

    #[test]
    fn token_response_maps_provider_error() {
        let token: OAuth2TokenResponse =
            serde_json::from_str(r#"{"error":"invalid_grant","error_description":"bad code"}"#)
                .expect("token");

        let error = token.require_access_token().expect_err("provider error");

        assert!(error.to_string().contains("invalid_grant"));
        assert!(error.to_string().contains("bad code"));
    }

    fn test_client(auth_url: &str, token_url: &str) -> OAuth2Client {
        let config = OAuth2Config::builder(auth_url, token_url, "client-1")
            .client_secret("secret-1")
            .scope("openid")
            .scope("email")
            .build()
            .expect("config");
        OAuth2Client::new(config).expect("client")
    }

    fn test_pkce() -> PkcePair {
        PkcePair {
            code_verifier: "verifier-1".to_owned(),
            code_challenge: "challenge-1".to_owned(),
            code_challenge_method: "S256".to_owned(),
        }
    }

    #[derive(Clone, Debug)]
struct CapturedRequest {
        path: String,
        body: String,
    }

    #[derive(Clone, Debug)]
struct TestResponse {
        status: u16,
        content_type: &'static str,
        body: String,
    }

    impl TestResponse {
        fn json(status: u16, body: &str) -> Self {
            Self {
                status,
                content_type: "application/json",
                body: body.to_owned(),
            }
        }
    }

    struct TestServer {
        address: String,
        handle: JoinHandle<Vec<CapturedRequest>>,
    }

    impl TestServer {
        fn spawn(responses: Vec<TestResponse>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
            let address = listener
                .local_addr()
                .expect("address should exist")
                .to_string();
            let handle = thread::spawn(move || {
                let mut requests = Vec::new();
                for response in responses {
                    let (mut stream, _) = listener.accept().expect("connection should arrive");
                    let request = read_request(&mut stream);
                    requests.push(request);
                    let payload = format!(
                        "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n{}",
                        response.status,
                        response.body.len(),
                        response.content_type,
                        response.body
                    );
                    stream
                        .write_all(payload.as_bytes())
                        .expect("response should write");
                }
                requests
            });
            Self { address, handle }
        }

        fn url(&self, path: &str) -> String {
            format!("http://{}{}", self.address, path)
        }

        fn finish(self) -> Vec<CapturedRequest> {
            self.handle.join().expect("server should join")
        }
    }

    fn read_request(stream: &mut std::net::TcpStream) -> CapturedRequest {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout should set");
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let read = stream.read(&mut chunk).expect("request should read");
            if read == 0 {
                break buffer.len();
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(end) = find_header_end(&buffer) {
                break end;
            }
        };
        let headers = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
        let content_length = parse_content_length(&headers);
        let full_len = header_end + 4 + content_length;
        while buffer.len() < full_len {
            let read = stream.read(&mut chunk).expect("body should read");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
        }

        let first_line = headers.lines().next().unwrap_or_default();
        let path = first_line
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_owned();
        let body = String::from_utf8_lossy(&buffer[header_end + 4..]).into_owned();
        CapturedRequest { path, body }
    }

    fn find_header_end(buffer: &[u8]) -> Option<usize> {
        buffer.windows(4).position(|window| window == b"\r\n\r\n")
    }

    fn parse_content_length(headers: &str) -> usize {
        headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }
}
