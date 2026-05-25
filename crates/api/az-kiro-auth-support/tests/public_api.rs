use az_derive_aliases::{apply, plain_clone_debug};
use az_kiro_auth_support::{
    BlockedCapability, EnglishNameOptions, KiroDeviceFlowClient, KiroLoginType, KiroOidcConfig,
    KiroTokenPoll, NameGender, PasswordPolicy, extract_verification_code, generate_english_name,
    generate_password, unsupported_capability,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[test]
fn kiro_device_flow_uses_oidc_paths_and_maps_poll_states() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"clientId":"client-1","clientSecret":"secret-1"}"#),
        TestResponse::json(
            r#"{"deviceCode":"device-1","userCode":"USER-CODE","verificationUriComplete":"https://verify.example.com/?user_code=USER-CODE","interval":1}"#,
        ),
        TestResponse::json(r#"{"error":"authorization_pending"}"#),
        TestResponse::json(
            r#"{"accessToken":"access-1","refreshToken":"refresh-1","tokenType":"Bearer","expiresIn":3600}"#,
        ),
    ])?;

    let client = KiroDeviceFlowClient::new(
        KiroOidcConfig::builder()
            .base_url(server.base_url())
            .poll_interval(Duration::from_millis(1))
            .poll_timeout(Duration::from_millis(50))
            .build()?,
    )?;
    let flow = client.begin_device_flow(KiroLoginType::Personal, None)?;

    assert_eq!(flow.user_code(), "USER-CODE");
    assert_eq!(
        flow.verification_url(),
        "https://verify.example.com/?user_code=USER-CODE"
    );
    assert!(matches!(
        client.poll_token_once(&flow, flow.poll_interval)?,
        KiroTokenPoll::Pending
    ));
    let poll = client.poll_token_once(&flow, flow.poll_interval)?;
    let KiroTokenPoll::Success(token) = poll else {
        panic!("expected success token poll");
    };
    assert_eq!(token.access_token.as_deref(), Some("access-1"));

    let requests = server.finish()?;
    assert_eq!(requests[0].path, "/client/register");
    assert!(
        requests[0]
            .body
            .contains("\"clientName\":\"Kiro Manual Auth\"")
    );
    assert!(requests[0].body.contains("codewhisperer:completions"));
    assert_eq!(requests[1].path, "/device_authorization");
    assert!(
        requests[1]
            .body
            .contains("\"startUrl\":\"https://view.awsapps.com/start\"")
    );
    assert_eq!(requests[2].path, "/token");
    assert!(
        requests[2]
            .body
            .contains("urn:ietf:params:oauth:grant-type:device_code")
    );
    Ok(())
}

#[test]
fn identity_helpers_generate_expected_shapes() -> Result<(), Box<dyn Error>> {
    let name = generate_english_name(EnglishNameOptions {
        full_name: true,
        gender: NameGender::Random,
    })?;
    let password = generate_password(PasswordPolicy {
        length: 16,
        symbols: "!".to_owned(),
    })?;

    assert!(name.display_name().contains(' '));
    assert_eq!(password.len(), 16);
    assert!(password.chars().any(|ch| ch.is_ascii_lowercase()));
    assert!(password.chars().any(|ch| ch.is_ascii_uppercase()));
    assert!(password.chars().any(|ch| ch.is_ascii_digit()));
    assert!(password.contains('!'));
    Ok(())
}

#[test]
fn verification_code_parser_matches_email_html() {
    assert_eq!(
        extract_verification_code(r#"<div class="code"> 778899 </div>"#).as_deref(),
        Some("778899")
    );
}

#[test]
fn unsupported_capabilities_return_explicit_error() {
    let error = unsupported_capability(BlockedCapability::AutomatedKiroRegistration)
        .expect_err("capability should be blocked")
        .to_string();

    assert!(error.contains("automated_kiro_registration"));
}

#[apply(plain_clone_debug)]
struct CapturedRequest {
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

#[apply(plain_clone_debug)]
struct TestResponse {
    status: u16,
    content_type: &'static str,
    body: String,
}

impl TestResponse {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "application/json",
            body: body.to_owned(),
        }
    }
}

struct TestServer {
    base_url: String,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
    handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl TestServer {
    fn spawn(responses: Vec<TestResponse>) -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let captured = Arc::new(Mutex::new(Vec::new()));
        let captured_clone = Arc::clone(&captured);

        let handle = thread::spawn(move || -> std::io::Result<()> {
            for response in responses {
                let (mut stream, _) = listener.accept()?;
                let request = read_request(&mut stream)?;
                captured_clone
                    .lock()
                    .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?
                    .push(request);
                write_response(&mut stream, response)?;
            }
            Ok(())
        });

        Ok(Self {
            base_url: format!("http://{address}"),
            captured,
            handle: Some(handle),
        })
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn finish(mut self) -> Result<Vec<CapturedRequest>, Box<dyn Error>> {
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(result) => result?,
                Err(_) => {
                    return Err(Box::new(std::io::Error::other(
                        "test server thread panicked",
                    )));
                }
            }
        }

        let guard = self
            .captured
            .lock()
            .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
        Ok(guard.clone())
    }
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<CapturedRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "request ended before headers",
            ));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(index) = find_bytes(&buffer, b"\r\n\r\n") {
            break index + 4;
        }
    };

    let header_text = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line")
    })?;
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing path"))?
        .to_owned();

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let normalized_name = name.trim().to_ascii_lowercase();
        let trimmed_value = value.trim().to_owned();
        if normalized_name == "content-length" {
            content_length = trimmed_value.parse::<usize>().unwrap_or_default();
        }
        headers.insert(normalized_name, trimmed_value);
    }

    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body = String::from_utf8_lossy(&buffer[header_end..header_end + content_length]).into();
    Ok(CapturedRequest {
        path,
        headers,
        body,
    })
}

fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
    let payload = response.body.into_bytes();
    write!(
        stream,
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.content_type,
        payload.len()
    )?;
    stream.write_all(&payload)?;
    stream.flush()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
