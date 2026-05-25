use az_sms::fivesim::{FivesimClient, FivesimConfig};
use az_sms::model::{SmsActivationRequest, SmsHostingRequest};
use az_sms::provider::SmsProvider;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[test]
fn fivesim_config_rejects_blank_token() {
    let err = FivesimConfig::builder(" ").build().unwrap_err();
    assert!(err.to_string().contains("api_token cannot be blank"));
}

#[tokio::test]
async fn fivesim_activation_url_adds_only_requested_query_options() {
    let (base_url, requests, server) = mock_server(vec![
        r#"{
          "id":11631253,
          "phone":"+447350690992",
          "operator":"any",
          "product":"telegram",
          "price":21,
          "status":"PENDING",
          "expires":"2018-10-13T08:28:38.809469028Z",
          "sms":[],
          "created_at":"2018-10-13T08:13:38.809469028Z",
          "forwarding":false,
          "forwarding_number":"",
          "country":"usa"
        }"#,
    ])
    .await;

    let request = SmsActivationRequest::new("usa", "any", "telegram")
        .unwrap()
        .reuse(true)
        .voice(false);
    let order = client(base_url)
        .buy_activation_number(request)
        .await
        .unwrap();
    server.await.unwrap();

    assert_eq!(order.id, 11631253);
    let line = first_request_line(&requests);
    assert!(line.starts_with("GET /v1/user/buy/activation/usa/any/telegram?"));
    assert!(line.contains("reuse=true"));
    assert!(line.contains("voice=false"));
}

#[tokio::test]
async fn fivesim_hosting_url_matches_provider_path() {
    let (base_url, requests, server) = mock_server(vec![
        r#"{
          "id":11631254,
          "phone":"+447350690993",
          "operator":"any",
          "product":"3hours",
          "price":12,
          "status":"PENDING",
          "expires":"2018-10-13T11:13:38.809469028Z",
          "sms":[],
          "created_at":"2018-10-13T08:13:38.809469028Z",
          "forwarding":false,
          "forwarding_number":"",
          "country":"usa"
        }"#,
    ])
    .await;

    let request = SmsHostingRequest::new("usa", "any", "3hours").unwrap();
    let order = client(base_url).buy_hosting_number(request).await.unwrap();
    server.await.unwrap();

    assert_eq!(order.id, 11631254);
    assert_eq!(
        first_request_line(&requests),
        "GET /v1/user/buy/hosting/usa/any/3hours HTTP/1.1"
    );
}

#[tokio::test]
async fn fivesim_provider_plain_text_errors_are_detected() {
    let (base_url, _requests, server) = mock_server(vec!["no free phones"]).await;

    let request = SmsActivationRequest::new("usa", "any", "telegram").unwrap();
    let err = client(base_url)
        .buy_activation_number(request)
        .await
        .unwrap_err();
    server.await.unwrap();

    assert!(err.to_string().contains("no free phones"));
}

fn client(base_url: String) -> FivesimClient {
    FivesimClient::new(
        FivesimConfig::builder("token")
            .base_url(base_url)
            .build()
            .unwrap(),
    )
    .unwrap()
}

async fn mock_server(
    responses: Vec<&'static str>,
) -> (String, Arc<Mutex<Vec<String>>>, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured_requests = requests.clone();

    let server = tokio::spawn(async move {
        for body in responses {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0_u8; 4096];
            let read = socket.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            let first_line = request.lines().next().unwrap_or_default().to_owned();
            captured_requests.lock().unwrap().push(first_line);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });

    (format!("http://{addr}/v1/"), requests, server)
}

fn first_request_line(requests: &Arc<Mutex<Vec<String>>>) -> String {
    requests
        .lock()
        .unwrap()
        .first()
        .cloned()
        .unwrap_or_default()
}
