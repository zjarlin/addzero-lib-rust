# az-mqtt

基于 rumqttc 封装的同步 MQTT 客户端，支持 TLS、发布/订阅、遗嘱消息和批量接收。

## 功能

- 同步的 MQTT 发布/订阅接口，内部维护后台轮询线程
- Builder 模式构建连接配置和消息
- TLS 支持：CA 证书 + 客户端证书双向认证
- QoS 0/1/2 三级服务质量
- 遗嘱消息（Last Will）
- `collect_messages()` 批量接收
- `Debug` 输出自动脱敏密码和私钥路径
- 全 crate 禁止 `unsafe`

## 安装

在 `Cargo.toml` 中添加：

\`\`\`toml
[dependencies]
az-mqtt = { path = "../mqtt" }          # workspace 内部引用
# 或发布后：
# az-mqtt = "0.1"                          # crates.io 引用
\`\`\`

## 用法

\`\`\`rust,no_run
use az_mqtt::api::{MqttClient, MqttConfig, MqttMessage, MqttQoS};
use std::time::Duration;

// 构建配置
let config = MqttConfig::builder("broker.example.com", "my-client")
    .port(1883)
    .build()
    .unwrap();

// 连接
let client = MqttClient::connect(config).unwrap();

// 订阅
client.subscribe("home/sensors/#", MqttQoS::AtLeastOnce).unwrap();

// 发布
client.publish_str("home/sensors/temp", "23.5", MqttQoS::AtMostOnce, false).unwrap();

// 接收（超时 5 秒）
if let Ok(Some(msg)) = client.receive_timeout(Duration::from_secs(5)) {
    println!("{}: {}", msg.topic, msg.payload_as_utf8_lossy());
}

// 断开
client.disconnect().unwrap();
\`\`\`

### TLS 连接

\`\`\`rust,no_run
use az_mqtt::api::{MqttClient, MqttConfig, MqttQoS};

let config = MqttConfig::builder("secure-broker.example.com", "tls-client")
    .port(8883)
    .username("user")
    .password("pass")
    .ca_path("/etc/ssl/ca.pem")
    .client_auth_paths("/etc/ssl/client.crt", "/etc/ssl/client.key")
    .build()
    .unwrap();

let client = MqttClient::connect(config).unwrap();
\`\`\`

## 依赖的 crates

- `rumqttc` — MQTT 3.1.1 协议客户端
- `rustls` — TLS 加密传输（ring 加密后端）
- `anyhow` — 错误链与上下文
