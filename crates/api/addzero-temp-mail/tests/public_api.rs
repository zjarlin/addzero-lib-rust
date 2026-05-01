use addzero_temp_mail::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[test]
fn temp_mail_create_mailbox_and_login_runs_full_flow() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"hydra:member":[{"id":"domain-1","domain":"mail.tm","isActive":true,"isPrivate":false}]}"#,
        ),
        TestResponse::json(r#"{"id":"account-1"}"#),
        TestResponse::json(r#"{"token":"token-1"}"#),
    ])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let mailbox = api.create_mailbox_and_login("az_", 10)?;

    assert!(mailbox.address.ends_with("@mail.tm"));
    assert_eq!(mailbox.account_id, "account-1");
    assert_eq!(mailbox.token, "token-1");
    assert_eq!(mailbox.password.len(), 10);

    let requests = server.finish()?;
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].path, "/domains");
    assert_eq!(requests[1].path, "/accounts");
    assert_eq!(requests[2].path, "/token");
    assert!(requests[1].body.contains("\"address\""));
    assert!(requests[2].body.contains("\"password\""));
    Ok(())
}

#[test]
fn temp_mail_get_message_flattens_html_array() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::json(
        r#"{"id":"msg-1","from":{"address":"from@mail.tm","name":"Sender"},"to":[{"address":"to@mail.tm","name":"Receiver"}],"subject":"Hello","text":"Plain","html":["<p>Hello</p>"],"createdAt":"2026-04-20T12:00:00.000Z"}"#,
    )])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let message = api.get_message("token-1", "msg-1")?;

    assert_eq!(message.id, "msg-1");
    assert_eq!(message.html, "<p>Hello</p>");
    assert_eq!(message.to.len(), 1);

    let requests = server.finish()?;
    let authorization = requests[0]
        .headers
        .get("authorization")
        .cloned()
        .ok_or_else(|| std::io::Error::other("missing authorization header"))?;
    assert_eq!(requests[0].path, "/messages/msg-1");
    assert_eq!(authorization, "Bearer token-1");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedRequest {
    method: String,
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
                let mut guard = captured_clone
                    .lock()
                    .map_err(|_| std::io::Error::other("request capture mutex poisoned"))?;
                guard.push(request);
                drop(guard);
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
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing method"))?
        .to_owned();
    let path = request_parts
        .next()
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
        method,
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
