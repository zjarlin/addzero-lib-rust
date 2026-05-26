use az_derive_aliases::{apply, plain_clone_debug};
use az_temp_mail::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[test]
fn cloudflare_context_builds_api_config_and_requests() -> Result<(), Box<dyn Error>> {
    let context = CloudflareTempMailContext {
        base_url: "https://mail.example.com".to_owned(),
        custom_auth: Some("admin-secret".to_owned()),
        address_name: Some("demo".to_owned()),
        address_domain: Some("example.com".to_owned()),
        cf_token: Some("cf-token".to_owned()),
        enable_random_subdomain: Some(true),
    };

    let config = context.api_config()?;
    let create_mailbox = context.create_mailbox_request();
    let new_address = context.new_address_request();

    assert_eq!(config.base_url, "https://mail.example.com");
    assert_eq!(
        config
            .default_headers
            .get("x-custom-auth")
            .map(String::as_str),
        Some("admin-secret")
    );
    assert_eq!(create_mailbox.name.as_deref(), Some("demo"));
    assert_eq!(create_mailbox.domain.as_deref(), Some("example.com"));
    assert_eq!(create_mailbox.cf_token.as_deref(), Some("cf-token"));
    assert!(create_mailbox.enable_random_subdomain);
    assert_eq!(new_address.name.as_deref(), Some("demo"));
    assert_eq!(new_address.domain.as_deref(), Some("example.com"));
    assert_eq!(new_address.cf_token.as_deref(), Some("cf-token"));
    assert_eq!(new_address.enable_random_subdomain, Some(true));

    Ok(())
}

#[test]
fn provider_kind_serializes_as_stable_code() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        serde_json::to_string(&TempMailProviderKind::Cloudflare)?,
        "\"cloudflare\""
    );
    assert_eq!(
        serde_json::from_str::<TempMailProviderKind>("\"mail_tm\"")?,
        TempMailProviderKind::MailTm
    );
    assert_eq!(TempMailProviderKind::Emailnator.as_str(), "emailnator");
    Ok(())
}

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

#[test]
fn temp_mail_provider_trait_supports_cloudflare_and_mail_tm() -> Result<(), Box<dyn Error>> {
    let cloudflare_server = TestServer::spawn(vec![
        TestResponse::json(r#"{"address":"demo@test.example.com","jwt":"jwt-1","address_id":7}"#),
        TestResponse::json(
            r#"{"results":[{"id":12,"source":"alice@test.example.com","address":"demo@test.example.com","raw":"Subject: Trait\r\n\r\nBody","created_at":"2026-05-09 10:00:00"}],"count":1}"#,
        ),
    ])?;
    let mail_tm_server = TestServer::spawn(vec![
        TestResponse::json(
            r#"{"hydra:member":[{"id":"domain-1","domain":"mail.tm","isActive":true,"isPrivate":false}]}"#,
        ),
        TestResponse::json(r#"{"id":"account-1"}"#),
        TestResponse::json(r#"{"token":"token-1"}"#),
        TestResponse::json(
            r#"{"hydra:member":[{"id":"msg-1","from":{"address":"from@mail.tm","name":"Sender"},"subject":"Hello","intro":"Intro","createdAt":"2026-05-09T10:00:00.000Z"}]}"#,
        ),
    ])?;

    let cloudflare =
        CloudflareTempMailApi::new(ApiConfig::builder(cloudflare_server.base_url()).build()?)?;
    let mail_tm = MailTmTempMailApi::new(ApiConfig::builder(mail_tm_server.base_url()).build()?)?;
    let providers: Vec<&dyn TempMailProvider> = vec![&cloudflare, &mail_tm];

    let cloudflare_mailbox =
        providers[0].create_mailbox(&CreateMailboxRequest::new("demo", "test.example.com"))?;
    let cloudflare_messages =
        providers[0].list_messages(&cloudflare_mailbox, PageRequest::new(10, 0))?;
    let mail_tm_mailbox = providers[1].create_mailbox(&CreateMailboxRequest::named("demo"))?;
    let mail_tm_messages = providers[1].list_messages(&mail_tm_mailbox, PageRequest::default())?;

    assert_eq!(
        providers[0].provider_kind(),
        TempMailProviderKind::Cloudflare
    );
    assert_eq!(cloudflare_messages.results[0].subject, "Trait");
    assert_eq!(providers[1].provider_kind(), TempMailProviderKind::MailTm);
    assert_eq!(mail_tm_messages.results[0].from_name, "Sender");

    let cloudflare_requests = cloudflare_server.finish()?;
    assert_eq!(cloudflare_requests[0].path, "/api/new_address");
    assert_eq!(cloudflare_requests[1].path, "/api/mails?limit=10&offset=0");
    let mail_tm_requests = mail_tm_server.finish()?;
    assert_eq!(mail_tm_requests[0].path, "/domains");
    assert_eq!(mail_tm_requests[1].path, "/accounts");
    assert_eq!(mail_tm_requests[2].path, "/token");
    assert_eq!(mail_tm_requests[3].path, "/messages?page=1");
    Ok(())
}

#[test]
fn temp_mail_provider_factory_builds_boxed_providers() -> Result<(), Box<dyn Error>> {
    let factory = BuiltinTempMailProviderFactory;

    let cloudflare_config =
        TempMailProviderConfig::Cloudflare(ApiConfig::builder("http://127.0.0.1:21001").build()?);
    assert_eq!(cloudflare_config.kind(), TempMailProviderKind::Cloudflare);
    let cloudflare = factory.build_provider(cloudflare_config)?;
    assert_eq!(cloudflare.provider_kind(), TempMailProviderKind::Cloudflare);

    let mail_tm = build_temp_mail_provider(TempMailProviderConfig::MailTm(
        ApiConfig::builder("http://127.0.0.1:21002").build()?,
    ))?;
    assert_eq!(mail_tm.provider_kind(), TempMailProviderKind::MailTm);

    let emailnator = factory.build_provider(TempMailProviderConfig::Emailnator(
        ApiConfig::builder("http://127.0.0.1:21003").build()?,
    ))?;
    assert_eq!(emailnator.provider_kind(), TempMailProviderKind::Emailnator);

    Ok(())
}

#[test]
fn emailnator_provider_uses_xsrf_cookie_and_message_paths() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::text("<html></html>").header("Set-Cookie", "XSRF-TOKEN=token%3D; Path=/"),
        TestResponse::json(r#"{"email":["demo@gmail.com"]}"#),
        TestResponse::json(
            r#"{"messageData":[{"messageID":"msg-1","from":"noreply@example.com","subject":"Code","time":"2026-05-11 10:00:00"}]}"#,
        ),
        TestResponse::text(r#"<div class="code">123456</div>"#),
    ])?;

    let api = EmailnatorTempMailApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let mailbox = api.create_mailbox(&CreateMailboxRequest::random())?;
    let messages = api.list_messages(&mailbox, PageRequest::default())?;
    let detail = api
        .get_message(&mailbox, &messages.results[0].id)?
        .expect("message detail");

    assert_eq!(mailbox.provider, TempMailProviderKind::Emailnator);
    assert_eq!(mailbox.address, "demo@gmail.com");
    assert_eq!(messages.results[0].id, "msg-1");
    assert!(detail.raw.contains("123456"));

    let requests = server.finish()?;
    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path, "/");
    assert_eq!(requests[1].path, "/generate-email");
    assert!(requests[1].body.contains("plusGmail"));
    assert_eq!(
        requests[1].headers.get("x-xsrf-token").map(String::as_str),
        Some("token=")
    );
    assert_eq!(
        requests[1].headers.get("cookie").map(String::as_str),
        Some("XSRF-TOKEN=token%3D")
    );
    assert_eq!(requests[2].path, "/message-list");
    assert!(requests[3].body.contains("\"messageID\":\"msg-1\""));
    Ok(())
}

#[apply(plain_clone_debug)]
struct CapturedRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: String,
}

#[apply(plain_clone_debug)]
struct TestResponse {
    status: u16,
    content_type: &'static str,
    headers: Vec<(&'static str, &'static str)>,
    body: String,
}

impl TestResponse {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "application/json",
            headers: Vec::new(),
            body: body.to_owned(),
        }
    }

    fn text(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "text/plain",
            headers: Vec::new(),
            body: body.to_owned(),
        }
    }

    fn header(mut self, name: &'static str, value: &'static str) -> Self {
        self.headers.push((name, value));
        self
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
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        response.content_type,
        payload.len()
    )?;
    for (name, value) in response.headers {
        write!(stream, "{name}: {value}\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(&payload)?;
    stream.flush()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
