use az_sms::grizzlysms::{GrizzlySmsClient, GrizzlySmsConfig};
use az_sms::model::{SmsActivationRequest, SmsOrderStatus};
use az_sms::provider::SmsProvider;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[test]
fn grizzlysms_config_rejects_blank_key() {
    let err = GrizzlySmsConfig::builder(" ").build().unwrap_err();
    assert!(err.to_string().contains("api_key cannot be blank"));
}

#[tokio::test]
async fn grizzlysms_buy_activation_number_maps_v2_response() {
    let (base_url, requests, server) = mock_server(vec![
        r#"{
          "activationCancel": "2026-05-07 14:03:16",
          "activationCost": 0.35,
          "activationEnd": "2026-05-07 14:18:16",
          "activationId": 495357953,
          "activationTime": "2026-05-07 13:58:16",
          "canGetAnotherSms": "0",
          "countryCode": "12",
          "currency": 643,
          "phoneNumber": "18036181752"
        }"#,
    ])
    .await;

    let client = client(base_url);
    let request = SmsActivationRequest::new("12", "any", "tg").unwrap();
    let order = client.buy_activation_number(request).await.unwrap();
    server.await.unwrap();

    assert_eq!(order.id, 495357953);
    assert_eq!(order.phone, "18036181752");
    assert_eq!(order.product, "tg");
    assert_eq!(order.status, SmsOrderStatus::Pending);
    assert_eq!(order.expires.as_deref(), Some("2026-05-07 14:18:16"));

    let line = first_request_line(&requests);
    assert!(line.contains("action=getNumberV2"));
    assert!(line.contains("api_key=key"));
    assert!(line.contains("service=tg"));
    assert!(line.contains("country=12"));
}

#[tokio::test]
async fn grizzlysms_check_order_enriches_received_sms() {
    let (base_url, _requests, server) = mock_server(vec![
        "STATUS_OK:852508",
        r#"[{
          "activationCost": 0.35,
          "activationId": "495367092",
          "activationStatus": 1,
          "activationTime": "2026-05-07 14:18:05",
          "countryName": "USA (2)",
          "phoneNumber": "19568190051",
          "serviceCode": "tg",
          "smsCode": "852508",
          "smsText": "Your code is 852508"
        }]"#,
        r#"{
          "verificationType": 2,
          "sms": {
            "dateTime": "2026-02-26 12:05:55",
            "code": "852508",
            "text": "Your code is 852508"
          }
        }"#,
    ])
    .await;

    let client = client(base_url);
    let order = client.check_order(495367092).await.unwrap();
    server.await.unwrap();

    assert_eq!(order.id, 495367092);
    assert_eq!(order.phone, "19568190051");
    assert_eq!(order.product, "tg");
    assert_eq!(order.status, SmsOrderStatus::Received);
    assert_eq!(
        order
            .sms
            .first()
            .and_then(|message| message.code.as_deref()),
        Some("852508")
    );
    assert_eq!(
        order
            .sms
            .first()
            .and_then(|message| message.date.as_deref()),
        Some("2026-02-26 12:05:55")
    );
}

#[tokio::test]
async fn grizzlysms_balance_parses_access_balance() {
    let (base_url, _requests, server) = mock_server(vec!["ACCESS_BALANCE:12.34"]).await;

    let balance = client(base_url).balance().await.unwrap();
    server.await.unwrap();

    assert_eq!(balance, 12.34);
}

#[tokio::test]
async fn grizzlysms_rejects_operator_specific_activation_requests() {
    let client = client("http://127.0.0.1:9/stubs/handler_api.php".to_owned());
    let request = SmsActivationRequest::new("12", "provider1", "tg").unwrap();

    let err = client.buy_activation_number(request).await.unwrap_err();

    assert!(err.to_string().contains("operator-specific"));
}

fn client(base_url: String) -> GrizzlySmsClient {
    GrizzlySmsClient::new(
        GrizzlySmsConfig::builder("key")
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

    (
        format!("http://{addr}/stubs/handler_api.php"),
        requests,
        server,
    )
}

fn first_request_line(requests: &Arc<Mutex<Vec<String>>>) -> String {
    requests
        .lock()
        .unwrap()
        .first()
        .cloned()
        .unwrap_or_default()
}
