use addzero_curl::*;
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

    let parsed = CurlParser::parse(command).expect("curl should parse");

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

    let parsed = CurlParser::parse(command).expect("curl should parse");

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
fn mutates_query_parameters_by_convention() {
    let updated = modify_existing_query_params(
        "https://example.com/api/users?id=42&page=3&status=ok&name=alice",
    )
    .expect("query params should mutate");

    assert!(updated.contains("id=invalid_id_123"));
    assert!(updated.contains("page=-1"));
    assert!(updated.contains("status=invalid_status"));
    assert!(updated.contains("name=modified_alice"));
}

#[test]
fn generates_rules_and_mutates_json_payload() {
    let payload = r#"{"name":"alice","age":18,"active":true,"items":[{"count":2}]}"#;

    let rules = generate_mutation_rules(payload).expect("rules should generate");
    let updated = mutate_payload(payload, &rules).expect("payload should mutate");

    assert_eq!(rules.get("name"), Some(&MutationRule::Number));
    assert_eq!(rules.get("age"), Some(&MutationRule::String));
    assert!(updated.contains("\"name\": 0"));
    assert!(updated.contains("\"age\": \"mutated_string\""));
    assert!(updated.contains("\"active\": null"));
    assert!(updated.contains("\"count\": \"mutated_string\""));
}

#[test]
fn update_payload_works_for_curl_command() {
    let command = r#"curl https://example.com/api -H 'content-type: application/json' -d '{"name":"alice","age":18}'"#;

    let updated = update_payload(command)
        .expect("payload mutation should succeed")
        .expect("payload should exist");

    assert!(updated.contains("\"name\": 0"));
    assert!(updated.contains("\"age\": \"mutated_string\""));
}

#[test]
fn executor_sends_request_to_local_server() {
    let (url, join_handle) = spawn_http_server("ok");
    let command = format!(
        "curl -X POST '{url}/echo?userId=42' -H 'x-token: abc' -H 'content-type: application/json' -d '{{\"hello\":\"world\"}}'"
    );

    let executor = CurlExecutor::new();
    let response = executor.execute(command).expect("request should succeed");
    let request = join_handle.join().expect("server thread should join");

    assert_eq!(response.status, 200);
    assert_eq!(response.text().expect("response should be text"), "ok");
    assert!(request.starts_with("POST /echo?userId=42 HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("x-token: abc"));
    assert!(request.contains("{\"hello\":\"world\"}"));
}

fn spawn_http_server(body: &'static str) -> (String, thread::JoinHandle<String>) {
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
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
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
