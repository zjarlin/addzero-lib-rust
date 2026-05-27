use crate::types::{ProxyError, ProxyResult, ProxyNode, ProxyType};
use serde_yaml::{Mapping, Value};

/// 从 Clash YAML 文档的 `proxies:` 列表中解析受支持的代理节点。
///
/// 不支持的代理类型会被忽略，因此混合配置中仍可提取可用节点。
///
/// # Errors
///
/// 当 YAML 无法解码、缺少 `proxies` 字段，或没有任何受支持代理项时返回错误。
pub fn parse_clash_yaml(input: &str) -> ProxyResult<Vec<ProxyNode>> {
    let document: Value = serde_yaml::from_str(input)?;
    let proxies = mapping_get(&document, "proxies")
        .and_then(Value::as_sequence)
        .ok_or(ProxyError::MissingField("proxies"))?;

    let mut nodes = Vec::new();
    for proxy in proxies {
        match parse_proxy_value(proxy) {
            Ok(Some(node)) => nodes.push(node),
            Ok(None) => {}
            Err(error) => tracing::debug!(%error, "skipped malformed clash proxy entry"),
        }
    }

    if nodes.is_empty() {
        return Err(ProxyError::NoUsableNodes);
    }

    Ok(nodes)
}

fn parse_proxy_value(value: &Value) -> ProxyResult<Option<ProxyNode>> {
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

fn get_str<'a>(mapping: &'a Mapping, key: &'static str) -> ProxyResult<&'a str> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .ok_or(ProxyError::MissingField(key))
}

fn get_port(mapping: &Mapping, key: &'static str) -> ProxyResult<u16> {
    let value = mapping
        .get(Value::String(key.to_owned()))
        .ok_or(ProxyError::MissingField(key))?;

    match value {
        Value::Number(number) => {
            let Some(port) = number.as_u64() else {
                return Err(ProxyError::InvalidPort(format!("{number:?}")));
            };
            u16::try_from(port).map_err(|_| ProxyError::InvalidPort(port.to_string()))
        }
        Value::String(port) => port
            .parse::<u16>()
            .map_err(|_| ProxyError::InvalidPort(port.clone())),
        _ => Err(ProxyError::InvalidPort(format!("{value:?}"))),
    }
}
