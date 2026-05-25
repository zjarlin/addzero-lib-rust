use az_proxy::clash::parse_clash_yaml;

#[test]
fn parse_clash_yaml_should_parse_supported_proxy_nodes() {
    let yaml = r#"
mixed-port: 7890
proxies:
  - name: "🇭🇰 香港 SS"
    type: ss
    server: hk.example.com
    port: 8388
    cipher: aes-128-gcm
    password: secret
  - name: "US VLESS"
    type: vless
    server: us.example.com
    port: "443"
    uuid: 00000000-0000-0000-0000-000000000000
  - name: "Unsupported"
    type: http
    server: example.com
    port: 8080
"#;

    let nodes = parse_clash_yaml(yaml).unwrap();

    assert_eq!(nodes.len(), 2);
}

#[test]
fn parse_clash_yaml_should_parse_country_from_flag() {
    let yaml = r#"
proxies:
  - name: "🇭🇰 香港 SS"
    type: ss
    server: hk.example.com
    port: 8388
"#;

    let nodes = parse_clash_yaml(yaml).unwrap();

    assert_eq!(nodes[0].country.as_deref(), Some("HK"));
}
