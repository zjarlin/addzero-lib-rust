use crate::fetcher::fetch_and_parse;
use crate::speedtest::batch_speed_test;
use crate::types::{
    ClashConfig, ClashError, ClashResult, DEFAULT_MIXED_PORT, DEFAULT_SPEEDTEST_TIMEOUT, ProxyNode,
    SpeedTestResult,
};
use serde_yaml::{Mapping, Number, Value};

/// Selects the fastest successful node according to sorted speed test results.
///
/// # Errors
///
/// Returns [`ClashError::NoSuccessfulSpeedTest`] when none of the results
/// succeeded or all successful results point outside the node slice.
pub fn select_fastest_node<'a>(
    nodes: &'a [ProxyNode],
    results: &[SpeedTestResult],
) -> ClashResult<&'a ProxyNode> {
    results
        .iter()
        .filter(|result| result.success)
        .filter_map(|result| nodes.get(result.node_index))
        .next()
        .ok_or(ClashError::NoSuccessfulSpeedTest)
}

/// Generates a minimal Clash YAML config containing `node` as the only proxy.
///
/// # Errors
///
/// Returns an error when the generated config cannot be serialized as YAML.
pub fn generate_clash_config(node: &ProxyNode, mixed_port: u16) -> ClashResult<String> {
    let proxy = normalized_proxy_value(node);
    let config = ClashConfig::minimal(mixed_port, proxy, node.name.clone());
    Ok(serde_yaml::to_string(&config)?)
}

/// Fetches, parses, speed-tests, selects the fastest node, and returns Clash YAML.
///
/// Uses [`DEFAULT_SPEEDTEST_TIMEOUT`] and [`DEFAULT_MIXED_PORT`].
///
/// # Errors
///
/// Returns an error when fetching/parsing fails, every speed test fails, or the
/// selected config cannot be serialized.
pub async fn select_fastest(url: &str, concurrency: usize) -> ClashResult<String> {
    let nodes = fetch_and_parse(url).await?;
    let results = batch_speed_test(&nodes, concurrency, DEFAULT_SPEEDTEST_TIMEOUT).await;
    let node = select_fastest_node(&nodes, &results)?;
    generate_clash_config(node, DEFAULT_MIXED_PORT)
}

fn normalized_proxy_value(node: &ProxyNode) -> Value {
    let Value::Mapping(mut map) = node.raw.clone() else {
        return fallback_proxy_value(node);
    };

    ensure_string(&mut map, "name", &node.name);
    ensure_string(&mut map, "type", node.node_type.as_clash_str());
    ensure_string(&mut map, "server", &node.server);
    ensure_port(&mut map, node.port);
    Value::Mapping(map)
}

fn fallback_proxy_value(node: &ProxyNode) -> Value {
    let mut map = Mapping::new();
    ensure_string(&mut map, "name", &node.name);
    ensure_string(&mut map, "type", node.node_type.as_clash_str());
    ensure_string(&mut map, "server", &node.server);
    ensure_port(&mut map, node.port);
    Value::Mapping(map)
}

fn ensure_string(map: &mut Mapping, key: &str, value: &str) {
    map.insert(
        Value::String(key.to_owned()),
        Value::String(value.to_owned()),
    );
}

fn ensure_port(map: &mut Mapping, port: u16) {
    map.insert(
        Value::String("port".to_owned()),
        Value::Number(Number::from(port)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProxyType;

    #[test]
    fn generate_clash_config_should_include_selected_proxy_group() {
        let node = ProxyNode::new(
            "Test Node",
            ProxyType::Ss,
            "127.0.0.1",
            8388,
            fallback_proxy_value(&ProxyNode {
                name: "Test Node".to_owned(),
                node_type: ProxyType::Ss,
                server: "127.0.0.1".to_owned(),
                port: 8388,
                country: None,
                raw: Value::Null,
            }),
        );

        let yaml = generate_clash_config(&node, 7890).unwrap();

        assert!(yaml.contains("proxy-groups:"));
    }
}
