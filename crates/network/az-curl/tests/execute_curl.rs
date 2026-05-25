use az_curl::execute::execute_curl;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn execute_curl_returns_response_body() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 2048];
        let _ = stream.read(&mut buffer).unwrap();

        let body = r#"{"ok":true}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let command =
        format!(r#"curl --location 'http://{addr}/api' --header 'accept: application/json'"#);
    let response = execute_curl(command).expect("local response should return a body");
    server.join().unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.text().unwrap(), r#"{"ok":true}"#);
}
