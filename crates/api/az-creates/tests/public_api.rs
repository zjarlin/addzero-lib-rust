use az_creates::{
    ApiConfig, BuiltinEmailSenderFactory, BuiltinSmsProviderFactory,
    BuiltinTempMailProviderFactory, Creates, DogSmsConfig, EmailConfig, EmailSenderConfig,
    EmailSenderFactory, EmailSenderKind, EmailnatorEmailMode, EmailnatorEmailRequest,
    GrizzlySmsConfig, MavenCentralApi, SmsProviderConfig, SmsProviderFactory, SmsProviderKind,
    TempMailApi, TempMailApiConfig, TempMailNewAddressRequest, TempMailPageRequest,
    TempMailProviderConfig, TempMailProviderFactory, TempMailProviderKind, extract_first_http_link,
};
use az_derive_aliases::{apply, plain_clone_debug};
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[test]
fn maven_search_parses_latest_version() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::json(
        r#"{"response":{"docs":[{"id":"com.google.guava:guava","g":"com.google.guava","a":"guava","latestVersion":"33.2.1-jre","p":"bundle","timestamp":123456}]}}"#,
    )])?;

    let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let artifacts = api.search_by_coordinates("com.google.guava", "guava", 5)?;

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].group_id, "com.google.guava");
    assert_eq!(artifacts[0].artifact_id, "guava");
    assert_eq!(artifacts[0].resolved_version(), Some("33.2.1-jre"));

    let requests = server.finish()?;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "GET");
    assert!(
        requests[0]
            .path
            .contains("/solrsearch/select?q=g%3Acom.google.guava+AND+a%3Aguava")
    );
    assert!(requests[0].path.contains("rows=5"));
    Ok(())
}

#[test]
fn maven_download_uses_remotecontent_endpoint() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![TestResponse::text("artifact-body")])?;

    let api = MavenCentralApi::new(ApiConfig::builder(server.base_url()).build()?)?;
    let bytes = api.download_file(
        "com.google.guava",
        "guava",
        "33.2.1-jre",
        "guava-33.2.1-jre.pom",
    )?;

    assert_eq!(String::from_utf8(bytes)?, "artifact-body");

    let requests = server.finish()?;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].path,
        "/remotecontent?filepath=com%2Fgoogle%2Fguava%2Fguava%2F33.2.1-jre%2Fguava-33.2.1-jre.pom"
    );
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

    let api = TempMailApi::new(TempMailApiConfig::builder(server.base_url()).build()?)?;
    let address = api.new_address(&TempMailNewAddressRequest::new("demo", "test.example.com"))?;
    let mails = api.list_parsed_mails(&address.jwt, TempMailPageRequest::new(10, 0))?;

    assert_eq!(address.address, "demo@test.example.com");
    assert_eq!(address.address_id, 7);
    assert_eq!(mails.count, 1);
    assert_eq!(mails.results[0].subject, "Hello");

    let requests = server.finish()?;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/api/new_address");
    assert!(requests[0].body.contains("\"name\":\"demo\""));
    assert_eq!(requests[1].path, "/api/parsed_mails?limit=10&offset=0");
    Ok(())
}

#[test]
fn temp_mail_password_helpers_hash_like_upstream_frontend() -> Result<(), Box<dyn Error>> {
    let server = TestServer::spawn(vec![
        TestResponse::json(r#"{"success":true}"#),
        TestResponse::json(r#"{"address":"demo@test.example.com","jwt":"jwt-2","address_id":7}"#),
    ])?;

    let api = TempMailApi::new(TempMailApiConfig::builder(server.base_url()).build()?)?;
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
fn creates_facade_builds_temp_mail_provider() -> Result<(), Box<dyn Error>> {
    let mail_tm_config = TempMailProviderConfig::MailTm(
        TempMailApiConfig::builder("http://127.0.0.1:22001").build()?,
    );
    assert_eq!(mail_tm_config.kind(), TempMailProviderKind::MailTm);
    let provider = Creates::temp_mail_provider(mail_tm_config)?;
    assert_eq!(provider.provider_kind(), TempMailProviderKind::MailTm);

    let factory: &dyn TempMailProviderFactory = &BuiltinTempMailProviderFactory;
    let emailnator = Creates::temp_mail_provider_with_factory(
        factory,
        TempMailProviderConfig::Emailnator(
            TempMailApiConfig::builder("http://127.0.0.1:22002").build()?,
        ),
    )?;
    assert_eq!(emailnator.provider_kind(), TempMailProviderKind::Emailnator);

    Ok(())
}

#[test]
fn creates_facade_builds_emailnator_client() -> Result<(), Box<dyn Error>> {
    let api = Creates::temp_mail_emailnator_with_config(
        TempMailApiConfig::builder("http://127.0.0.1:22003").build()?,
    )?;
    drop(api);

    let request = EmailnatorEmailRequest::new([EmailnatorEmailMode::DotGmail]);
    assert_eq!(request.modes, vec![EmailnatorEmailMode::DotGmail]);

    assert_eq!(
        extract_first_http_link("code: https://example.com/verify", Some("verify")).as_deref(),
        Some("https://example.com/verify")
    );

    Ok(())
}

#[test]
fn creates_facade_builds_sms_provider() -> Result<(), Box<dyn Error>> {
    let dogsms = DogSmsConfig::builder("token").build()?;
    assert_eq!(
        SmsProviderConfig::from(dogsms.clone()).kind(),
        SmsProviderKind::DogSms
    );

    let provider = Creates::sms_provider(dogsms.into())?;
    drop(provider);

    let factory: &dyn SmsProviderFactory = &BuiltinSmsProviderFactory;
    let grizzly = GrizzlySmsConfig::builder("api-key").build()?;
    let provider = Creates::sms_provider_with_factory(factory, grizzly.into())?;
    drop(provider);

    Ok(())
}

#[test]
fn creates_facade_builds_email_sender() -> Result<(), Box<dyn Error>> {
    let config = EmailConfig::builder("smtp.example.com", "user@example.com", "secret").build()?;
    let sender_config = EmailSenderConfig::from(config.clone());
    assert_eq!(sender_config.kind(), EmailSenderKind::Smtp);
    assert_eq!(EmailSenderKind::Smtp.code(), "smtp");

    let sender = Creates::email_sender(sender_config)?;
    drop(sender);

    let factory: &dyn EmailSenderFactory = &BuiltinEmailSenderFactory;
    let sender = Creates::email_sender_with_factory(factory, config.clone().into())?;
    drop(sender);

    let sender = Creates::smtp_email_with_factory(factory, config)?;
    drop(sender);

    Ok(())
}

#[apply(plain_clone_debug)]
struct CapturedRequest {
    method: String,
    path: String,
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

    fn text(body: &str) -> Self {
        Self {
            status: 200,
            content_type: "text/plain; charset=utf-8",
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
                Ok(result) => {
                    result?;
                }
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
            let kind = std::io::ErrorKind::UnexpectedEof;
            let error = std::io::Error::new(kind, "request ended before headers");

            return Err(error);
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
    }

    while buffer.len() < header_end + content_length {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }

    let body_bytes = if content_length == 0 {
        &[][..]
    } else {
        &buffer[header_end..header_end + content_length]
    };

    Ok(CapturedRequest {
        method,
        path,
        body: String::from_utf8_lossy(body_bytes).into_owned(),
    })
}

fn write_response(stream: &mut TcpStream, response: TestResponse) -> std::io::Result<()> {
    let body = response.body;
    let payload = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        response.content_type,
        body.len(),
        body
    );
    stream.write_all(payload.as_bytes())?;
    stream.flush()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
