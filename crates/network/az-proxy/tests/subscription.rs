use az_proxy::parser::parse_subscription;
use base64::Engine;

#[test]
fn parse_subscription_should_parse_base64_uri_lines() {
    let body = base64::engine::general_purpose::STANDARD.encode(
        "vless://00000000-0000-0000-0000-000000000000@example.com:443?type=ws&security=tls#Test",
    );

    let nodes = parse_subscription(&body, None).unwrap();

    assert_eq!(nodes.len(), 1);
}
