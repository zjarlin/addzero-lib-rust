use az_mqtt::client::MqttConfig;

#[test]
fn mqtt_config_debug_does_not_leak_credentials() {
    let config = MqttConfig::builder("localhost", "client-1")
        .username("alice")
        .password("mqtt-secret")
        .ca_path("/tmp/ca.pem")
        .client_auth_paths("/tmp/client.crt", "/tmp/client.key")
        .build()
        .expect("mqtt config should build");

    let output = format!("{config:?}");
    assert!(output.contains("localhost"));
    assert!(!output.contains("mqtt-secret"));
    assert!(!output.contains("/tmp/client.key"));
    assert!(output.contains("/tmp/ca.pem"));
    assert!(output.contains("/tmp/client.crt"));
}
