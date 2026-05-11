# az-clash

Clash 订阅解析、代理节点测试和最小配置生成。

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
az-clash = { path = "../az-clash" }       # workspace 内部引用
# 或发布后：
# az-clash = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_clash::select_fastest;

// 从订阅 URL 获取、解析、测速并生成最优配置
let config = select_fastest("https://example.com/sub", 10).await?;
println!("{config}");
```

```rust
use az_clash::{fetch_and_parse, batch_speed_test, generate_clash_config};

// 分步操作
let nodes = fetch_and_parse("https://example.com/sub").await?;
let results = batch_speed_test(&nodes, 10).await;
let config = generate_clash_config(&results);
```

## 依赖的 crates

- `automod` - 自动模块声明
- `base64` - Base64 编解码（部分订阅内容为 Base64 编码）
- `reqwest` - HTTP 客户端，用于获取订阅
- `serde` / `serde_json` / `serde_yaml` - 序列化与 YAML/JSON 解析
- `thiserror` - 错误类型派生
- `tokio` - 异步运行时
- `url` / `urlencoding` - URL 解析与编码
- `tracing` - 日志追踪
