use az_codex_auth_support::{DuckMailApi, DuckMailConfig};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[test]
fn duckmail_create_random_mailbox_uses_domains_accounts_and_token() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"hydra:member":[{"id":"domain-1","domain":"duckmail.sbs","isVerified":true}]}"#,
        ),
        TestResponse::json(
            r#"{"id":"account-1","address":"created@duckmail.sbs","authType":"email"}"#,
        ),
        TestResponse::json(r#"{"id":"account-1","token":"mail-token-1"}"#),
    ])?;

    let api = DuckMailApi::new(
        DuckMailConfig::builder(server.base_url())
            .auth_token("dk_test")
            .build()?,
    )?;
    let mailbox = api.create_random_mailbox_and_login(None)?;

    assert!(mailbox.address.ends_with("@duckmail.sbs"));
    assert_eq!(mailbox.account_id, "account-1");
    assert_eq!(mailbox.token, "mail-token-1");

    let requests = server.finish()?;
    assert_eq!(requests[0].path, "/domains?page=1");
    assert_eq!(requests[1].path, "/accounts");
    assert_eq!(requests[2].path, "/token");
    assert_eq!(
        requests[0].headers.get("authorization").map(String::as_str),
        Some("Bearer dk_test")
    );
    assert_eq!(
        requests[1].headers.get("authorization").map(String::as_str),
        Some("Bearer dk_test")
    );
    assert!(requests[1].body.contains("\"address\""));
    assert!(requests[2].body.contains("\"password\""));
    Ok(())
}

#[test]
fn duckmail_messages_read_body_from_html_array() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"hydra:member":[{"id":"msg-1","from":{"name":"OpenAI","address":"noreply@example.com"},"subject":"Code"}]}"#,
        ),
        TestResponse::json(
            r#"{"id":"msg-1","from":{"name":"OpenAI","address":"noreply@example.com"},"to":[],"subject":"Code","text":"","html":["<p>Verification code: 123456</p>"]}"#,
        ),
    ])?;

    let api = DuckMailApi::new(DuckMailConfig::builder(server.base_url()).build()?)?;
    let messages = api.list_messages("mail-token-1", 1)?;
    let detail = api.get_message("mail-token-1", &messages[0].id)?;

    assert_eq!(messages[0].id, "msg-1");
    assert_eq!(detail.body(), "<p>Verification code: 123456</p>");

    let requests = server.finish()?;
    assert_eq!(requests[0].path, "/messages?page=1");
    assert_eq!(requests[1].path, "/messages/msg-1");
    assert_eq!(
        requests[0].headers.get("authorization").map(String::as_str),
        Some("Bearer mail-token-1")
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedRequest {
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

#[derive(Debug, Clone)]
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
