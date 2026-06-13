# az-config-center-client

配置中心 Rust 同步客户端，语义对齐 Kotlin Multiplatform SDK：先登录，再选择命名空间，然后按配置键读取或写入运行配置。

## 功能

- 基于 `reqwest::blocking` 的同步客户端，不要求调用方引入 Tokio runtime
- 复用 `az-config-center-contract` 中的 API DTO，避免 Rust client、后端和其他 SDK 字段漂移
- 支持 `login`、`checkout_namespace`、`status`、`list`、`get_item`
- 支持文本、密钥、整数、浮点数、布尔和 JSON 对象读写
- 缺失或停用配置在 `get_*` 方法中返回 `Ok(None)`

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-config-center-client = { path = "../az-config-center-client" }
```

## 用法

```rust,no_run
use az_config_center_client::client::ConfigCenterClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RedisConfig {
    host: String,
    port: u16,
}

let client = ConfigCenterClient::new("http://127.0.0.1:8080")?
    .login("admin", std::env::var("CONFIG_CENTER_PASSWORD").unwrap_or_default())?
    .checkout_namespace("cmp-aio.dev")?;

let app_name = client.get_text("app.name")?;
let enabled = client.get_bool("feature.enabled")?;
let redis: Option<RedisConfig> = client.get_json("redis")?;

client.set_text("app.name", "cmp-aio", "应用名")?;
client.set_bool("feature.enabled", true, "功能开关")?;
client.set_json(
    "redis",
    &RedisConfig {
        host: "127.0.0.1".to_owned(),
        port: 6379,
    },
    "Redis 连接配置",
)?;
# Ok::<(), anyhow::Error>(())
```

## 依赖的 crates

- `az-config-center-contract` — 配置中心共享请求和响应 DTO
- `anyhow` — 错误上下文与统一 `Result`
- `reqwest` — HTTP 客户端（阻塞模式）
- `serde` / `serde_json` — JSON 序列化与反序列化
