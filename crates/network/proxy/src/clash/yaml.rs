use crate::types::{ProxyNode, ProxyType};
use anyhow::{Context, Result, anyhow, bail};
use serde_yaml::{Mapping, Value};

/// 从 Clash YAML 文档的 `proxies:` 列表中解析受支持的代理节点。
///
/// 不支持的代理类型会被忽略，因此混合配置中仍可提取可用节点。
///
/// # Errors
///
/// 当 YAML 无法解码、缺少 `proxies` 字段，或没有任何受支持代理项时返回错误。
pub fn parse_clash_yaml(input: &str) -> Result<Vec<ProxyNode>> {
    let document: Value = serde_yaml::from_str(input).context("parse Clash YAML subscription")?;
    let proxies = mapping_get(&document, "proxies")
        .and_then(Value::as_sequence)
        .context("missing required field `proxies`")?;

    let mut nodes = Vec::new();
    for proxy in proxies {
        match parse_proxy_value(proxy) {
            Ok(Some(node)) => nodes.push(node),
            Ok(None) => {}
            Err(error) => tracing::debug!(%error, "skipped malformed clash proxy entry"),
        }
    }

    if nodes.is_empty() {
        bail!("subscription did not contain usable proxy nodes");
    }

    Ok(nodes)
}

fn parse_proxy_value(value: &Value) -> Result<Option<ProxyNode>> {
    let Some(mapping) = value.as_mapping() else {
        return Ok(None);
    };

    let proxy_type_value = get_str(mapping, "type")?;
    let Some(node_type) = ProxyType::from_clash_type(proxy_type_value) else {
        tracing::debug!(
            proxy_type = proxy_type_value,
            "skipped unsupported proxy type"
        );
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

fn get_str<'a>(mapping: &'a Mapping, key: &'static str) -> Result<&'a str> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing required field `{key}`"))
}

fn get_port(mapping: &Mapping, key: &'static str) -> Result<u16> {
    let value = mapping
        .get(Value::String(key.to_owned()))
        .ok_or_else(|| anyhow!("missing required field `{key}`"))?;

    match value {
        Value::Number(number) => {
            let Some(port) = number.as_u64() else {
                bail!("invalid proxy port `{number:?}`");
            };
            u16::try_from(port).with_context(|| format!("invalid proxy port `{port}`"))
        }
        Value::String(port) => port
            .parse::<u16>()
            .with_context(|| format!("invalid proxy port `{port}`")),
        _ => bail!("invalid proxy port `{value:?}`"),
    }
}
