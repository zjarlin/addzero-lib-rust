use az_proxy::speedtest::batch_speed_test;
use az_proxy::types::{ProxyNode, ProxyType};
use serde_yaml::Value;
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn batch_speed_test_should_report_success_for_open_tcp_port() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accept_task = tokio::spawn(async move {
        let _accepted = listener.accept().await;
    });
    let node = ProxyNode::new("Local", ProxyType::Ss, "127.0.0.1", port, Value::Null);

    let results = batch_speed_test(&[node], 1, Duration::from_secs(1)).await;
    let _ = accept_task.await;

    assert!(results[0].success);
}
