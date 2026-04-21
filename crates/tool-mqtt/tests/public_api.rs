use addzero_mqtt::*;

#[test]
fn mqtt_message_builder_validates_topic() {
    let error = MqttMessage::builder("   ").build().unwrap_err();
    assert!(matches!(error, MqttError::InvalidMessage(_)));
}

#[test]
fn mqtt_config_builder_validates_auth_pairing() {
    let error = MqttConfig::builder("localhost", "client-a")
        .password("secret")
        .build()
        .unwrap_err();
    assert!(matches!(error, MqttError::InvalidConfig(_)));
}

#[test]
fn mqtt_config_builder_persists_runtime_fields() {
    let last_will = MqttMessage::builder("system/last-will")
        .payload_str("bye")
        .qos(MqttQoS::AtLeastOnce)
        .retain(true)
        .build()
        .unwrap();
    let config = MqttConfig::builder("broker.example.com", "client-a")
        .port(2883)
        .username("user")
        .password("pass")
        .keep_alive_secs(30)
        .clean_session(false)
        .request_channel_capacity(32)
        .inflight(7)
        .last_will(last_will)
        .build()
        .unwrap();

    assert_eq!(config.host, "broker.example.com");
    assert_eq!(config.port, 2883);
    assert_eq!(config.client_id, "client-a");
    assert_eq!(config.username.as_deref(), Some("user"));
    assert_eq!(config.password.as_deref(), Some("pass"));
    assert_eq!(config.keep_alive_secs, 30);
    assert!(!config.clean_session);
    assert_eq!(config.request_channel_capacity, 32);
    assert_eq!(config.inflight, 7);

    let last_will = config.last_will.expect("last will should exist");
    assert_eq!(last_will.topic, "system/last-will");
    assert_eq!(last_will.payload, b"bye".to_vec());
    assert_eq!(last_will.qos, MqttQoS::AtLeastOnce);
    assert!(last_will.retain);
}

#[test]
fn mqtt_config_builder_persists_tls_flag() {
    let config = MqttConfig::builder("broker.example.com", "client-a")
        .use_tls(true)
        .build()
        .unwrap();

    assert!(config.use_tls);
    assert_eq!(config.ca_path, None);
    assert_eq!(config.client_cert_path, None);
    assert_eq!(config.client_key_path, None);
}

#[test]
fn mqtt_config_builder_persists_custom_tls_paths() {
    let config = MqttConfig::builder("broker.example.com", "client-a")
        .ca_path("/tmp/ca.pem")
        .client_auth_paths("/tmp/client.pem", "/tmp/client.key")
        .build()
        .unwrap();

    assert!(config.use_tls);
    assert_eq!(config.ca_path.as_deref(), Some("/tmp/ca.pem"));
    assert_eq!(config.client_cert_path.as_deref(), Some("/tmp/client.pem"));
    assert_eq!(config.client_key_path.as_deref(), Some("/tmp/client.key"));
}
