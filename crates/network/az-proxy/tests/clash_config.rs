use az_proxy::clash::generate_clash_config;
use az_proxy::types::{ProxyNode, ProxyType};

#[test]
fn generate_clash_config_should_include_selected_proxy_group() {
    let raw = serde_yaml::from_str(
        r#"
name: Test Node
type: ss
server: 127.0.0.1
port: 8388
"#,
    )
    .unwrap();
    let node = ProxyNode::new("Test Node", ProxyType::Ss, "127.0.0.1", 8388, raw);

    let yaml = generate_clash_config(&node, 7890).unwrap();

    assert!(yaml.contains("proxy-groups:"));
}
