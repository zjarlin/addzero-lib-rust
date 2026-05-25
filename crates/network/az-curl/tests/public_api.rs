use az_curl::*;
use reqwest::Method;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

#[test]
fn parses_complex_post_command() {
    let command = r#"
        curl 'https://demo.jetlinks.cn/api/device-product/_query' \
          -H 'accept: application/json, text/plain, */*' \
          -H 'content-type: application/json' \
          -H 'x-access-token: token-123' \
          --data-raw '{"pageIndex":0,"pageSize":96,"sorts":[{"name":"createTime","order":"desc"}],"terms":[]}'
    "#;

    let result = parse_curl(command);
    let parsed = result.expect("curl should parse");

    assert_eq!(parsed.method, Method::POST);
    assert_eq!(
        parsed.url,
        "https://demo.jetlinks.cn/api/device-product/_query"
    );
    assert_eq!(
        parsed.header("accept"),
        Some("application/json, text/plain, */*")
    );
    assert_eq!(parsed.header("x-access-token"), Some("token-123"));
    assert_eq!(parsed.inferred_content_type(), Some("application/json"));
    assert!(
        parsed
            .body
            .as_deref()
            .expect("body should exist")
            .contains("\"pageIndex\":0")
    );
}

#[test]
fn parses_auth_query_and_form_data() {
    let command = "curl --url https://example.com/api/v1/users/42/orders/a1b2c3d4e5?userId=42&page=2 -u demo:secret -F 'name=alice' -F 'type=premium'";

    let parsed = parse_curl(command).expect("curl should parse");

    assert_eq!(parsed.method, Method::POST);
    assert_eq!(
        parsed.query_params.get("userId").map(String::as_str),
        Some("42")
    );
    assert_eq!(
        parsed.query_params.get("page").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        parsed.path_params,
        vec!["42".to_owned(), "a1b2c3d4e5".to_owned()]
    );
    assert_eq!(
        parsed.form_params.get("name").map(String::as_str),
        Some("alice")
    );
    assert_eq!(parsed.inferred_content_type(), Some("multipart/form-data"));
    assert!(
        parsed
            .authorization
            .as_deref()
            .expect("authorization should exist")
            .starts_with("Basic ")
    );
}

#[test]
fn executor_sends_request_to_local_server() {
    let (url, join_handle) = spawn_http_server("ok");
    let command = format!(
        "curl -X POST '{url}/echo?userId=42' -H 'x-token: abc' -H 'content-type: application/json' -d '{{\"hello\":\"world\"}}'"
    );

    let executor = CurlExecutor::new();
    let result = executor.execute(command);
    let response = result.expect("request should succeed");
    let request = join_handle.join().expect("server thread should join");

    assert_eq!(response.status, 200);
    assert_eq!(response.text().expect("response should be text"), "ok");
    assert!(request.starts_with("POST /echo?userId=42 HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("x-token: abc"));
    assert!(request.contains("{\"hello\":\"world\"}"));
}

#[test]
fn executor_returns_unauthorized_response_body() {
    let response_body = r#"{"message":"用户未登录","result":{"text":"用户未登录","value":"expired"},"status":401,"code":"unauthorized","timestamp":1779679276215}"#;
    let (url, join_handle) = spawn_http_server_with_status(401, response_body);
    let command = format!(
        "curl -X POST '{url}/api/device-product/_query' -H 'content-type: application/json' --data-raw '{{\"pageIndex\":0,\"pageSize\":96,\"sorts\":[{{\"name\":\"createTime\",\"order\":\"desc\"}}],\"terms\":[]}}'"
    );

    let response = CurlExecutor::new()
        .execute(command)
        .expect("401 response should still return a body");
    let _request = join_handle.join().expect("server thread should join");

    assert_eq!(response.status, 401);
    // 401 JSON must stay available as the response body, not be replaced by the request payload.
    assert_eq!(
        response.text().expect("response body should be utf-8"),
        response_body
    );
}

#[test]
fn parse_reports_missing_flag_values_as_structured_errors() {
    let error = parse_curl("curl --header")
        .expect_err("missing header value should fail")
        .to_string();

    assert_eq!(error, "flag `--header` requires a value");
}

fn spawn_http_server(body: &'static str) -> (String, thread::JoinHandle<String>) {
    spawn_http_server_with_status(200, body)
}

fn spawn_http_server_with_status(
    status: u16,
    body: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let address = listener.local_addr().expect("address should exist");

    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should arrive");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout should set");

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
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
        let full_length = header_end + 4 + content_length;

        while buffer.len() < full_length {
            let read = stream.read(&mut chunk).expect("body should read");
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
        }

        let response = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("response should write");

        String::from_utf8_lossy(&buffer).into_owned()
    });

    (format!("http://{address}"), handle)
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
