use az_proxy::types::ProxyType;

#[test]
fn proxy_type_uses_clash_wire_codes() {
    assert_eq!(ProxyType::Ss.as_clash_str(), "ss");
    assert_eq!(ProxyType::Hysteria2.as_clash_str(), "hysteria2");
    assert_eq!(
        ProxyType::from_clash_type("shadowsocks"),
        Some(ProxyType::Ss)
    );
    assert_eq!(
        ProxyType::from_clash_type("hy2"),
        Some(ProxyType::Hysteria2)
    );
    assert_eq!(ProxyType::from_clash_type("VMESS"), Some(ProxyType::Vmess));
}

#[test]
fn proxy_type_serializes_as_snake_case_code() {
    assert_eq!(
        serde_json::to_string(&ProxyType::Wireguard).expect("proxy type should serialize"),
        "\"wireguard\""
    );
}
