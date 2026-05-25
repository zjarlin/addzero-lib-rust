use az_proxy::parser::parse_proxy_uri;
use az_proxy::types::ProxyType;
use base64::Engine;

#[test]
fn parse_proxy_uri_should_parse_vless_uri() {
    let node = parse_proxy_uri(
        "vless://00000000-0000-0000-0000-000000000000@example.com:443?type=ws&encryption=none&host=cdn.example.com&path=%2Fedge&security=tls&sni=sni.example.com#%F0%9F%87%AD%F0%9F%87%B0%20香港节点",
    )
    .unwrap();

    assert_eq!(node.node_type, ProxyType::Vless);
    assert_eq!(node.server, "example.com");
    assert_eq!(node.port, 443);
    assert_eq!(node.country.as_deref(), Some("HK"));
}

#[test]
fn parse_proxy_uri_should_parse_vmess_uri() {
    let json = r#"{
        "v": "2",
        "ps": "US VMess",
        "add": "vmess.example.com",
        "port": "443",
        "id": "00000000-0000-0000-0000-000000000000",
        "aid": "0",
        "scy": "auto",
        "net": "ws",
        "host": "cdn.example.com",
        "path": "/ws",
        "tls": "tls",
        "sni": "sni.example.com"
    }"#;
    let uri = format!(
        "vmess://{}",
        base64::engine::general_purpose::STANDARD.encode(json)
    );

    let node = parse_proxy_uri(&uri).unwrap();

    assert_eq!(node.node_type, ProxyType::Vmess);
    assert_eq!(node.server, "vmess.example.com");
}

#[test]
fn parse_proxy_uri_should_parse_ss_uri() {
    let userinfo = base64::engine::general_purpose::STANDARD.encode("aes-128-gcm:secret");
    let uri = format!("ss://{userinfo}@ss.example.com:8388#Japan");

    let node = parse_proxy_uri(&uri).unwrap();

    assert_eq!(node.node_type, ProxyType::Ss);
    assert_eq!(node.port, 8388);
}
