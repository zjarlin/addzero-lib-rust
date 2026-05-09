use az_temp_mail::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[test]
fn temp_mail_create_address_and_list_parsed_mail_use_cloudflare_worker_paths()
-> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"address":"demo@test.example.com","jwt":"jwt-1","password":"pw-1","address_id":7}"#,
        ),
        TestResponse::json(
            r#"{"results":[{"id":11,"address":"demo@test.example.com","sender":"Alice <alice@test.example.com>","subject":"Hello","text":"Plain","html":"<p>Hello</p>","attachments":[{"filename":"a.txt","mimeType":"text/plain","disposition":"attachment","size":5}],"created_at":"2026-05-09 10:00:00"}],"count":1}"#,
        ),
    ])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let address = api.new_address(&NewAddressRequest::new("demo", "test.example.com"))?;
    let mails = api.list_parsed_mails(&address.jwt, PageRequest::new(10, 0))?;

    assert_eq!(address.address, "demo@test.example.com");
    assert_eq!(address.address_id, 7);
    assert_eq!(mails.count, 1);
    assert_eq!(mails.results[0].subject, "Hello");
    assert_eq!(mails.results[0].attachments[0].mime_type, "text/plain");

    let requests = server.finish()?;
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].path, "/api/new_address");
    assert!(requests[0].body.contains("\"name\":\"demo\""));
    assert_eq!(requests[1].method, "GET");
    assert_eq!(requests[1].path, "/api/parsed_mails?limit=10&offset=0");
    assert_eq!(
        requests[1].headers.get("authorization").map(String::as_str),
        Some("Bearer jwt-1")
    );
    Ok(())
}

#[test]
fn temp_mail_password_helpers_hash_like_upstream_frontend() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"success":true}"#),
        TestResponse::json(r#"{"address":"demo@test.example.com","jwt":"jwt-2","address_id":7}"#),
    ])?;

    let api = TempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let changed = api.change_plain_password("jwt-1", "secret")?;
    let login = api.address_login_plain_password("demo@test.example.com", "secret")?;

    assert!(changed.success);
    assert_eq!(login.jwt, "jwt-2");

    let requests = server.finish()?;
    let expected_hash = "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b";
    assert_eq!(requests[0].path, "/api/address_change_password");
    assert!(requests[0].body.contains(expected_hash));
    assert_eq!(requests[1].path, "/api/address_login");
    assert!(requests[1].body.contains(expected_hash));
    Ok(())
}

#[test]
fn temp_mail_send_mail_and_mutations_use_bearer_auth() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"status":"ok"}"#),
        TestResponse::json(r#"{"success":true}"#),
    ])?;

    let api = create_temp_mail_api(server.base_url())?;
    let sent = api.send_mail(
        "jwt-1",
        &SendMailRequest::html("recipient@test.example.com", "Subject", "<p>Body</p>")
            .from_name("Sender")
            .to_name("Recipient"),
    )?;
    let cleared = api.clear_inbox("jwt-1")?;

    assert_eq!(sent.status.as_deref(), Some("ok"));
    assert!(cleared.success);

    let requests = server.finish()?;
    assert_eq!(requests[0].path, "/api/send_mail");
    assert!(requests[0].body.contains("\"is_html\":true"));
    assert_eq!(requests[1].method, "DELETE");
    assert_eq!(requests[1].path, "/api/clear_inbox");
    for request in requests {
        assert_eq!(
            request.headers.get("authorization").map(String::as_str),
            Some("Bearer jwt-1")
        );
    }
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
