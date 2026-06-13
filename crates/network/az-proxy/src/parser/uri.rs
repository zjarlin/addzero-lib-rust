use crate::parser::decode_base64_text;
use crate::types::{ProxyNode, ProxyType};
use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::{Mapping, Number, Value};
use std::collections::BTreeMap;
use url::Url;

/// 将单个受支持的代理 URI 解析为代理节点。
///
/// 支持的 URI scheme 包括 `ss`、`vmess`、`vless`、`trojan`、
/// `hysteria2`、`hy2`、`tuic` 和 `wireguard`。
///
/// # Errors
///
/// 当 scheme 不受支持，或 URI 缺少生成 Clash 兼容代理项所需的数据时返回错误。
pub fn parse_proxy_uri(input: &str) -> Result<ProxyNode> {
    let input = input.trim();
    let scheme = input
        .split_once("://")
        .map(|(scheme, _)| scheme.to_ascii_lowercase())
        .with_context(|| format!("invalid proxy uri: {input}"))?;

    match scheme.as_str() {
        "ss" => parse_ss_uri(input),
        "vmess" => parse_vmess_uri(input),
        "vless" => parse_url_like_uri(input, ProxyType::Vless),
        "trojan" => parse_url_like_uri(input, ProxyType::Trojan),
        "hysteria2" | "hy2" => parse_url_like_uri(input, ProxyType::Hysteria2),
        "tuic" => parse_url_like_uri(input, ProxyType::Tuic),
        "wireguard" => parse_url_like_uri(input, ProxyType::Wireguard),
        _ => bail!("unsupported proxy type `{scheme}`"),
    }
}

/// 解析按行分隔的代理 URI 订阅。
///
/// 空行和不包含受支持 URI scheme 的行会被忽略。
///
/// # Errors
///
/// 当某个可识别 URI 行格式错误，或最终没有任何可用节点时返回错误。
pub fn parse_uri_lines(input: &str) -> Result<Vec<ProxyNode>> {
    let mut nodes = Vec::new();
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if !has_supported_scheme(line) {
            tracing::debug!(line, "skipped subscription line without supported proxy scheme");
            continue;
        }
        nodes.push(parse_proxy_uri(line)?);
    }

    if nodes.is_empty() {
        bail!("subscription did not contain usable proxy nodes");
    }

    Ok(nodes)
}

fn has_supported_scheme(line: &str) -> bool {
    matches!(
        line.split_once("://").map(|(scheme, _)| scheme),
        Some("ss" | "vmess" | "vless" | "trojan" | "hysteria2" | "hy2" | "tuic" | "wireguard")
    )
}

fn parse_url_like_uri(input: &str, node_type: ProxyType) -> Result<ProxyNode> {
    let url = Url::parse(input).with_context(|| format!("parse proxy uri `{input}`"))?;
    let server = url
        .host_str()
        .context("missing required field `server`")?
        .to_owned();
    let port = url.port().context("missing required field `port`")?;
    let query = query_pairs(&url);
    let name = decoded_fragment(&url).unwrap_or_else(|| server.clone());
    let raw = raw_from_url_like(&url, node_type, &name, &server, port, &query);

    Ok(ProxyNode::new(name, node_type, server, port, raw))
}

fn parse_vmess_uri(input: &str) -> Result<ProxyNode> {
    let payload = input
        .strip_prefix("vmess://")
        .with_context(|| format!("invalid proxy uri: {input}"))?;
    let decoded = decode_base64_text(payload)
        .context("invalid proxy uri: vmess payload is not valid base64 text")?;
    let json: JsonValue =
        serde_json::from_str(decoded.trim()).context("parse vmess uri json payload")?;

    let server = json_str(&json, "add")
        .context("missing required field `add`")?
        .to_owned();
    let port = json_port(&json, "port")?;
    let name = json_str(&json, "ps").unwrap_or(&server).to_owned();
    let raw = raw_from_vmess_json(&json, &name, &server, port);

    Ok(ProxyNode::new(
        name,
        ProxyType::Vmess,
        server,
        port,
        raw,
    ))
}

fn parse_ss_uri(input: &str) -> Result<ProxyNode> {
    let payload = input
        .strip_prefix("ss://")
        .with_context(|| format!("invalid proxy uri: {input}"))?;
    let (without_fragment, fragment) = split_once(payload, '#');
    let (main, query) = split_once(without_fragment, '?');
    let decoded_main = if main.contains('@') {
        main.to_owned()
    } else {
        decode_base64_text(main)
            .context("invalid proxy uri: ss payload is not valid base64")?
    };

    let (userinfo, server_port) = decoded_main
        .rsplit_once('@')
        .context("invalid proxy uri: ss uri is missing user info")?;
    let userinfo = decode_ss_userinfo(userinfo)?;
    let (cipher, password) = userinfo
        .split_once(':')
        .context("invalid proxy uri: ss user info is missing cipher")?;
    let (server, port) = parse_server_port(server_port)?;
    let name = fragment
        .filter(|value| !value.is_empty())
        .map(decode_component)
        .unwrap_or_else(|| server.clone());
    let query = query
        .map(parse_raw_query)
        .unwrap_or_default();
    let raw = raw_from_ss(&name, &server, port, cipher, password, &query);

    Ok(ProxyNode::new(name, ProxyType::Ss, server, port, raw))
}

fn raw_from_url_like(
    url: &Url,
    node_type: ProxyType,
    name: &str,
    server: &str,
    port: u16,
    query: &BTreeMap<String, String>,
) -> Value {
    let mut map = base_proxy_mapping(name, node_type, server, port);
    let username = decode_component(url.username());
    let password = url.password().map(decode_component);

    match node_type {
        ProxyType::Vless => {
            insert_non_empty(&mut map, "uuid", &username);
            insert_non_empty(&mut map, "encryption", query.get("encryption").map_or("none", String::as_str));
            if let Some(network) = query.get("type").or_else(|| query.get("network")) {
                insert_non_empty(&mut map, "network", network);
                insert_transport_options(&mut map, network, query);
            }
            insert_tls_options(&mut map, query, true);
            insert_non_empty_from_query(&mut map, "flow", query, "flow");
        }
        ProxyType::Trojan => {
            insert_non_empty(&mut map, "password", &username);
            insert_tls_options(&mut map, query, true);
            if let Some(network) = query.get("type").or_else(|| query.get("network")) {
                insert_non_empty(&mut map, "network", network);
                insert_transport_options(&mut map, network, query);
            }
        }
        ProxyType::Hysteria2 => {
            insert_non_empty(&mut map, "password", &username);
            insert_non_empty_from_query(&mut map, "sni", query, "sni");
            insert_non_empty_from_query(&mut map, "obfs", query, "obfs");
            insert_non_empty_from_query(&mut map, "obfs-password", query, "obfs-password");
        }
        ProxyType::Tuic => {
            insert_non_empty(&mut map, "uuid", &username);
            if let Some(password) = password.as_deref() {
                insert_non_empty(&mut map, "password", password);
            }
            insert_non_empty_from_query(&mut map, "sni", query, "sni");
            insert_non_empty_from_query(&mut map, "congestion-controller", query, "congestion-controller");
            insert_non_empty_from_query(&mut map, "udp-relay-mode", query, "udp-relay-mode");
        }
        ProxyType::Wireguard => {
            for (key, value) in query {
                insert_non_empty(&mut map, key, value);
            }
        }
        ProxyType::Ss | ProxyType::Vmess => {}
    }

    Value::Mapping(map)
}

fn raw_from_vmess_json(json: &JsonValue, name: &str, server: &str, port: u16) -> Value {
    let mut map = base_proxy_mapping(name, ProxyType::Vmess, server, port);
    if let Some(uuid) = json_str(json, "id") {
        insert_non_empty(&mut map, "uuid", uuid);
    }
    if let Some(cipher) = json_str(json, "scy").filter(|value| !value.is_empty()) {
        insert_non_empty(&mut map, "cipher", cipher);
    } else {
        insert_non_empty(&mut map, "cipher", "auto");
    }
    insert_json_u64(&mut map, "alterId", json, "aid", 0);

    if let Some(network) = json_str(json, "net").filter(|value| !value.is_empty()) {
        insert_non_empty(&mut map, "network", network);
        let host = json_str(json, "host");
        let path = json_str(json, "path");
        insert_ws_opts(&mut map, network, host, path);
    }

    let tls = json_str(json, "tls").is_some_and(|value| value.eq_ignore_ascii_case("tls"));
    if tls {
        map.insert(Value::String("tls".to_owned()), Value::Bool(true));
    }
    if let Some(servername) = json_str(json, "sni").or_else(|| json_str(json, "host")) {
        insert_non_empty(&mut map, "servername", servername);
    }

    Value::Mapping(map)
}

fn raw_from_ss(
    name: &str,
    server: &str,
    port: u16,
    cipher: &str,
    password: &str,
    query: &BTreeMap<String, String>,
) -> Value {
    let mut map = base_proxy_mapping(name, ProxyType::Ss, server, port);
    insert_non_empty(&mut map, "cipher", cipher);
    insert_non_empty(&mut map, "password", password);
    for (key, value) in query {
        insert_non_empty(&mut map, key, value);
    }
    Value::Mapping(map)
}

fn base_proxy_mapping(name: &str, node_type: ProxyType, server: &str, port: u16) -> Mapping {
    let mut map = Mapping::new();
    insert_non_empty(&mut map, "name", name);
    insert_non_empty(&mut map, "type", node_type.as_clash_str());
    insert_non_empty(&mut map, "server", server);
    map.insert(
        Value::String("port".to_owned()),
        Value::Number(Number::from(port)),
    );
    map
}

fn insert_transport_options(
    map: &mut Mapping,
    network: &str,
    query: &BTreeMap<String, String>,
) {
    let host = query.get("host").map(String::as_str);
    let path = query.get("path").map(String::as_str);
    insert_ws_opts(map, network, host, path);
}

fn insert_ws_opts(map: &mut Mapping, network: &str, host: Option<&str>, path: Option<&str>) {
    if !network.eq_ignore_ascii_case("ws") {
        return;
    }

    let mut opts = Mapping::new();
    if let Some(path) = path.filter(|value| !value.is_empty()) {
        insert_non_empty(&mut opts, "path", path);
    }

    if let Some(host) = host.filter(|value| !value.is_empty()) {
        let mut headers = Mapping::new();
        insert_non_empty(&mut headers, "Host", host);
        opts.insert(
            Value::String("headers".to_owned()),
            Value::Mapping(headers),
        );
    }

    if !opts.is_empty() {
        map.insert(
            Value::String("ws-opts".to_owned()),
            Value::Mapping(opts),
        );
    }
}

fn insert_tls_options(map: &mut Mapping, query: &BTreeMap<String, String>, default_tls: bool) {
    let tls = query
        .get("security")
        .is_some_and(|value| value.eq_ignore_ascii_case("tls"))
        || query
            .get("tls")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
        || default_tls && query.get("security").is_some_and(|value| value.eq_ignore_ascii_case("tls"));

    if tls {
        map.insert(Value::String("tls".to_owned()), Value::Bool(true));
    }

    if let Some(sni) = query.get("sni") {
        insert_non_empty(map, "servername", sni);
    }
}

fn insert_non_empty_from_query(
    map: &mut Mapping,
    yaml_key: &str,
    query: &BTreeMap<String, String>,
    query_key: &str,
) {
    if let Some(value) = query.get(query_key) {
        insert_non_empty(map, yaml_key, value);
    }
}

fn insert_non_empty(map: &mut Mapping, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    map.insert(
        Value::String(key.to_owned()),
        Value::String(value.to_owned()),
    );
}

fn insert_json_u64(
    map: &mut Mapping,
    yaml_key: &str,
    json: &JsonValue,
    json_key: &str,
    default: u64,
) {
    let value = json
        .get(json_key)
        .and_then(json_value_as_u64)
        .unwrap_or(default);
    map.insert(
        Value::String(yaml_key.to_owned()),
        Value::Number(Number::from(value)),
    );
}

fn query_pairs(url: &Url) -> BTreeMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn parse_raw_query(input: &str) -> BTreeMap<String, String> {
    url::form_urlencoded::parse(input.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn decoded_fragment(url: &Url) -> Option<String> {
    url.fragment()
        .filter(|fragment| !fragment.is_empty())
        .map(decode_component)
}

fn decode_component(value: &str) -> String {
    urlencoding::decode(value)
        .map(|decoded| decoded.into_owned())
        .unwrap_or_else(|_| value.to_owned())
}

fn split_once(input: &str, delimiter: char) -> (&str, Option<&str>) {
    input
        .split_once(delimiter)
        .map_or((input, None), |(left, right)| (left, Some(right)))
}

fn decode_ss_userinfo(input: &str) -> Result<String> {
    let decoded = decode_component(input);
    if decoded.contains(':') {
        return Ok(decoded);
    }

    decode_base64_text(input)
        .context("invalid proxy uri: ss user info is not method:password or base64")
}

fn parse_server_port(input: &str) -> Result<(String, u16)> {
    let (server, port) = if let Some(rest) = input.strip_prefix('[') {
        let (server, rest) = rest
            .split_once(']')
            .context("invalid proxy uri: invalid bracketed ipv6 host")?;
        let port = rest
            .strip_prefix(':')
            .context("invalid proxy uri: missing port after ipv6 host")?;
        (server.to_owned(), port)
    } else {
        let (server, port) = input
            .rsplit_once(':')
            .context("invalid proxy uri: missing server port separator")?;
        (server.to_owned(), port)
    };

    let port = port
        .parse::<u16>()
        .with_context(|| format!("invalid proxy port `{port}`"))?;
    Ok((server, port))
}

fn json_str<'a>(json: &'a JsonValue, key: &str) -> Option<&'a str> {
    json.get(key).and_then(JsonValue::as_str)
}

fn json_port(json: &JsonValue, key: &'static str) -> Result<u16> {
    let value = json
        .get(key)
        .ok_or_else(|| anyhow!("missing required field `{key}`"))?;
    let port = json_value_as_u64(value)
        .ok_or_else(|| anyhow!("invalid proxy port `{}`", format_json_value(value)))?;
    u16::try_from(port).with_context(|| format!("invalid proxy port `{port}`"))
}

fn json_value_as_u64(value: &JsonValue) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|value| value.parse::<u64>().ok()))
}

fn format_json_value<T: Serialize>(value: T) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "<unprintable>".to_owned())
}
