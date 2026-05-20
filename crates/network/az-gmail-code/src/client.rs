use crate::config::GmailCodeQuery;
use crate::model::{ExtractedGmailCode, GmailListMessagesResponse, GmailMessage};
use crate::parser::{collect_message_body_candidates, extract_verification_code};
use crate::{GmailCodeConfig, GmailCodeError, GmailCodeResult};
use reqwest::Url;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;

/// Blocking Gmail API client for reading verification codes from an authorized mailbox.
#[derive(Debug, Clone)]
pub struct GmailCodeClient {
    base_url: Url,
    user_id: String,
    access_token: String,
    client: Client,
}

impl GmailCodeClient {
    /// Creates a client with default Gmail API settings and the required OAuth access token.
    pub fn new(access_token: impl Into<String>) -> GmailCodeResult<Self> {
        Self::with_config(GmailCodeConfig::builder(access_token).build()?)
    }

    /// Creates a client from explicit configuration.
    pub fn with_config(config: GmailCodeConfig) -> GmailCodeResult<Self> {
        config.validate()?;
        let base_url = Url::parse(&config.base_url)
            .map_err(|_| GmailCodeError::InvalidBaseUrl(config.base_url.clone()))?;

        let mut builder = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout);
        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        Ok(Self {
            base_url,
            user_id: config.user_id,
            access_token: config.access_token,
            client: builder.build()?,
        })
    }

    /// Searches Gmail messages using the query object and returns matching message ids.
    pub fn list_message_ids(&self, query: &GmailCodeQuery) -> GmailCodeResult<Vec<String>> {
        let response = self.list_messages(query)?;
        Ok(response
            .messages
            .into_iter()
            .map(|message| message.id)
            .collect())
    }

    /// Fetches one Gmail message by id with `format=full`.
    pub fn get_message(&self, message_id: impl AsRef<str>) -> GmailCodeResult<GmailMessage> {
        let message_id = message_id.as_ref();
        let path = format!(
            "users/{}/messages/{}",
            urlencoding::encode(&self.user_id),
            urlencoding::encode(message_id)
        );
        let response = self
            .authenticated_get(&path)?
            .query(&[("format", "full")])
            .send()?;
        Self::read_json(response)
    }

    /// Finds the latest verification code from messages matching the query.
    ///
    /// Gmail returns list results newest-first for typical searches, so this method
    /// inspects messages in listed order and returns the first code found.
    pub fn find_latest_code(
        &self,
        query: GmailCodeQuery,
    ) -> GmailCodeResult<Option<ExtractedGmailCode>> {
        let response = self.list_messages(&query)?;
        for summary in response.messages {
            let message = self.get_message(&summary.id)?;
            if let Some(code) = extract_code_from_message(&message)? {
                return Ok(Some(code));
            }
        }
        Ok(None)
    }

    fn list_messages(&self, query: &GmailCodeQuery) -> GmailCodeResult<GmailListMessagesResponse> {
        let path = format!("users/{}/messages", urlencoding::encode(&self.user_id));
        let gmail_q = query.gmail_q();
        let mut request = self
            .authenticated_get(&path)?
            .query(&[("maxResults", query.max_results.to_string())]);

        if !gmail_q.is_empty() {
            request = request.query(&[("q", gmail_q)]);
        }
        for label_id in &query.label_ids {
            if !label_id.trim().is_empty() {
                request = request.query(&[("labelIds", label_id)]);
            }
        }

        Self::read_json(request.send()?)
    }

    fn authenticated_get(&self, path: &str) -> GmailCodeResult<RequestBuilder> {
        Ok(self
            .client
            .get(self.join_url(path)?)
            .bearer_auth(&self.access_token))
    }

    fn read_json<T: DeserializeOwned>(response: Response) -> GmailCodeResult<T> {
        let response = Self::ensure_success(response)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }

    fn ensure_success(response: Response) -> GmailCodeResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let url = response.url().to_string();
        let body = match response.bytes() {
            Ok(bytes) => String::from_utf8_lossy(bytes.as_ref()).into_owned(),
            Err(error) => return Err(GmailCodeError::Transport(error)),
        };
        Err(GmailCodeError::HttpStatus {
            url,
            status: status.as_u16(),
            body,
        })
    }

    fn join_url(&self, path: &str) -> GmailCodeResult<Url> {
        self.base_url
            .join(path)
            .map_err(|_| GmailCodeError::InvalidPath(path.to_owned()))
    }
}

fn extract_code_from_message(
    message: &GmailMessage,
) -> GmailCodeResult<Option<ExtractedGmailCode>> {
    for candidate in collect_message_body_candidates(message)? {
        if let Some(code) = extract_verification_code(&candidate.text) {
            return Ok(Some(ExtractedGmailCode {
                code,
                message_id: message.id.clone(),
                thread_id: message.thread_id.clone(),
                from: message.header("From").map(ToOwned::to_owned),
                subject: message.header("Subject").map(ToOwned::to_owned),
                source_mime_type: candidate.mime_type,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::GmailCodeClient;
    use crate::{GmailCodeConfig, GmailCodeQuery};
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn find_latest_code_searches_and_fetches_message() {
        let server = MockGmailServer::spawn(vec![
            MockResponse::json(
                200,
                r#"{"messages":[{"id":"msg-1","threadId":"th-1"}],"resultSizeEstimate":1}"#,
            ),
            MockResponse::json(
                200,
                &format!(
                    r#"{{
                        "id":"msg-1",
                        "threadId":"th-1",
                        "payload":{{
                            "mimeType":"multipart/alternative",
                            "headers":[
                                {{"name":"From","value":"security@example.com"}},
                                {{"name":"Subject","value":"Login verification"}}
                            ],
                            "parts":[
                                {{
                                    "partId":"1",
                                    "mimeType":"text/plain",
                                    "body":{{"data":"{}"}}
                                }}
                            ]
                        }}
                    }}"#,
                    encode("Your verification code: 246810")
                ),
            ),
        ]);
        let client = test_client(server.base_url());

        let code = client
            .find_latest_code(
                GmailCodeQuery::new()
                    .from("security@example.com")
                    .subject("Login verification")
                    .newer_than("10m")
                    .unread(true),
            )
            .expect("code lookup")
            .expect("code should exist");

        assert_eq!(code.code, "246810");
        assert_eq!(code.message_id, "msg-1");
        assert_eq!(code.from.as_deref(), Some("security@example.com"));
        assert_eq!(code.subject.as_deref(), Some("Login verification"));

        let requests = server.requests();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("GET /gmail/v1/users/me/messages?"));
        assert!(requests[0].contains("maxResults=10"));
        assert!(requests[0].contains("labelIds=INBOX"));
        assert!(
            requests[0]
                .to_ascii_lowercase()
                .contains("authorization: bearer test-token")
        );
        assert!(requests[1].starts_with("GET /gmail/v1/users/me/messages/msg-1?format=full"));
    }

    #[test]
    fn http_status_keeps_response_body() {
        let server = MockGmailServer::spawn(vec![MockResponse::text(401, "bad token")]);
        let client = test_client(server.base_url());

        let error = client
            .list_message_ids(&GmailCodeQuery::new())
            .expect_err("401 should fail");

        assert!(error.to_string().contains("HTTP 401"));
        assert!(error.to_string().contains("bad token"));
    }

    fn test_client(base_url: String) -> GmailCodeClient {
        let config = GmailCodeConfig::builder("test-token")
            .base_url(base_url)
            .request_timeout(Duration::from_secs(3))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .expect("config should build");
        GmailCodeClient::with_config(config).expect("client should build")
    }

    fn encode(value: &str) -> String {
        URL_SAFE_NO_PAD.encode(value.as_bytes())
    }

    #[derive(Debug, Clone)]
    struct MockResponse {
        status: u16,
        content_type: &'static str,
        body: String,
    }

    impl MockResponse {
        fn json(status: u16, body: &str) -> Self {
            Self {
                status,
                content_type: "application/json",
                body: body.to_owned(),
            }
        }

        fn text(status: u16, body: &str) -> Self {
            Self {
                status,
                content_type: "text/plain",
                body: body.to_owned(),
            }
        }
    }

    struct MockGmailServer {
        base_url: String,
        requests: Arc<Mutex<Vec<String>>>,
        handle: thread::JoinHandle<()>,
    }

    impl MockGmailServer {
        fn spawn(responses: Vec<MockResponse>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
            let address = listener.local_addr().expect("address should exist");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let thread_requests = Arc::clone(&requests);
            let handle = thread::spawn(move || {
                for response in responses {
                    let (mut stream, _) = listener.accept().expect("connection should arrive");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("timeout should set");

                    let request = read_http_request(&mut stream);
                    thread_requests.lock().expect("requests lock").push(request);

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
            });

            Self {
                base_url: format!("http://{address}/gmail/v1/"),
                requests,
                handle,
            }
        }

        fn base_url(&self) -> String {
            self.base_url.clone()
        }

        fn requests(self) -> Vec<String> {
            self.handle.join().expect("server should finish");
            Arc::try_unwrap(self.requests)
                .expect("server requests should be owned")
                .into_inner()
                .expect("requests lock")
        }
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let read = stream.read(&mut chunk).expect("request should read");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
            if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8_lossy(&buffer).into_owned()
    }
}
