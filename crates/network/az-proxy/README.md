# az-proxy

代理订阅解析、节点测速和 Clash/Mihomo 最小配置生成。Clash 是 `az_proxy::clash` 下的一个模块，不再作为整个 crate 的名字。

## 功能

- 获取远程 Clash 订阅 URL 并解析返回内容
- 解析 Clash YAML 格式和常见代理 URI（SS、SSR、VMess、Trojan、VLESS、Hysteria 等）
- 批量测试代理节点 TCP 延迟
- 为选定的最快节点生成最小可用 Clash 配置
- 不依赖 Clash Verge 或任何 Clash 二进制文件

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-proxy = { path = "../az-proxy" }       # workspace 内部引用
# 或发布后：
# az-proxy = "0.1"                        # crates.io 引用
```

## 用法

```rust,no_run
use az_proxy::clash::select_fastest;

# async fn run() -> anyhow::Result<()> {
// 从订阅 URL 获取、解析、测速并生成最优配置
let config = select_fastest("https://example.com/sub", 10).await?;
println!("{config}");
# Ok(())
# }
```

```rust,no_run
use az_proxy::clash::generate_clash_config;
use az_proxy::fetcher::fetch_and_parse;
use az_proxy::selector::select_fastest_node;
use az_proxy::speedtest::batch_speed_test;
use az_proxy::types::DEFAULT_SPEEDTEST_TIMEOUT;

# async fn run() -> anyhow::Result<()> {
// 分步操作
let nodes = fetch_and_parse("https://example.com/sub").await?;
let results = batch_speed_test(&nodes, 10, DEFAULT_SPEEDTEST_TIMEOUT).await;
let selected = select_fastest_node(&nodes, &results)?;
let config = generate_clash_config(selected, 7890)?;
# Ok(())
# }
```

## 依赖的 crates

- `automod` - 自动模块声明
- `base64` - Base64 编解码（部分订阅内容为 Base64 编码）
- `reqwest` - HTTP 客户端，用于获取订阅
- `serde` / `serde_json` / `serde_yaml` - 序列化与 YAML/JSON 解析
- `anyhow` - 错误链与上下文
- `tokio` - 异步运行时
- `url` / `urlencoding` - URL 解析与编码
- `tracing` - 日志追踪
