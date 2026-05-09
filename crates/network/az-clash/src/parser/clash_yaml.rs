use crate::types::{ClashError, ClashResult, ProxyNode, ProxyType};
use serde_yaml::{Mapping, Value};

/// Parses supported proxy nodes from the `proxies:` list of a Clash YAML document.
///
/// Unsupported proxy types are ignored so a mixed Clash config can still yield
/// usable nodes.
///
/// # Errors
///
/// Returns an error when the YAML cannot be decoded, the `proxies` field is
/// absent, or no supported proxy entries can be parsed.
pub fn parse_clash_yaml(input: &str) -> ClashResult<Vec<ProxyNode>> {
    let document: Value = serde_yaml::from_str(input)?;
    let proxies = mapping_get(&document, "proxies")
        .and_then(Value::as_sequence)
        .ok_or(ClashError::MissingField("proxies"))?;

    let mut nodes = Vec::new();
    for proxy in proxies {
        match parse_proxy_value(proxy) {
            Ok(Some(node)) => nodes.push(node),
            Ok(None) => {}
            Err(error) => tracing::debug!(%error, "skipped malformed clash proxy entry"),
        }
    }

    if nodes.is_empty() {
        return Err(ClashError::NoUsableNodes);
    }

    Ok(nodes)
}

fn parse_proxy_value(value: &Value) -> ClashResult<Option<ProxyNode>> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(None);
    };

    let proxy_type_value = get_str(mapping, "type")?;
    let Some(node_type) = ProxyType::from_clash_type(proxy_type_value) else {
        tracing::debug!(proxy_type = proxy_type_value, "skipped unsupported proxy type");
        return Ok(None);
    };

    let name = get_str(mapping, "name")?.to_owned();
    let server = get_str(mapping, "server")?.to_owned();
    let port = get_port(mapping, "port")?;

    Ok(Some(ProxyNode::new(
        name,
        node_type,
        server,
        port,
        value.clone(),
    )))
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn get_str<'a>(mapping: &'a Mapping, key: &'static str) -> ClashResult<&'a str> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .ok_or(ClashError::MissingField(key))
}

fn get_port(mapping: &Mapping, key: &'static str) -> ClashResult<u16> {
    let value = mapping
        .get(Value::String(key.to_owned()))
        .ok_or(ClashError::MissingField(key))?;

    match value {
        Value::Number(number) => {
            let Some(port) = number.as_u64() else {
                return Err(ClashError::InvalidPort(format!("{number:?}")));
            };
            u16::try_from(port).map_err(|_| ClashError::InvalidPort(port.to_string()))
        }
        Value::String(port) => port
            .parse::<u16>()
            .map_err(|_| ClashError::InvalidPort(port.clone())),
        _ => Err(ClashError::InvalidPort(format!("{value:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
