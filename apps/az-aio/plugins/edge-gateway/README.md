# az-aio-plugin-edge-gateway

Edge Gateway 插件 — 提供网关流编辑器、运行时执行和参考模板，用于在 AIO Desktop 中编排和调试 HTTP 调用链。

## 功能

- **网关流设计**：支持多步骤 HTTP 调用链（plan），定义请求顺序、Header 模板和响应捕获
- **运行时执行**：基于 `reqwest` + Handlebars 模板引擎，按顺序执行网关步骤，支持前置步骤结果注入
- **JSONPath 捕获**：每一步可使用 JSONPath 从响应中提取值，供后续步骤引用
- **模板参考**：内置 Session Proxy、Login + Profile Chain 等参考模板
- **AIO Desktop 集成**：通过 `az-desktop-plugin` 注册为桌面插件，提供页面视图、工具栏操作和摘要卡片

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-plugin-edge-gateway = { path = "../az-aio-plugin-edge-gateway" }  # workspace 内部引用
# 或发布后：
# az-aio-plugin-edge-gateway = "2026.5.10"                                # crates.io 引用
```

## 用法

本 crate 是一个桌面插件，由 AIO Desktop 通过 plugin registry 自动发现和加载。以下为参考示例（网关计划执行）：

```rust,no_run
use az_aio_plugin_edge_gateway::gateway_runtime::run_gateway_plan;
use az_aio_plugin_edge_gateway::gateway_runtime_types::{GatewayRunRequest, GatewayRuntimeStep};
use serde_json::Value;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let plan = GatewayRunRequest {
        entry_route: "/edge/session-proxy".to_string(),
        input: Value::Null,
        steps: vec![GatewayRuntimeStep {
            body_preview: String::new(),
            capture_path: "$.headers.host".to_string(),
            depends_on: Vec::new(),
            headers: BTreeMap::new(),
            id: "ping".to_string(),
            input_refs: Vec::new(),
            kind: "curl".to_string(),
            label: "GET postman echo".to_string(),
            method: "GET".to_string(),
            notes: "Reference flow".to_string(),
            url: "https://postman-echo.com/get?source=aio-desktop".to_string(),
        }],
    };

    let result = run_gateway_plan(plan).await?;
    println!("status={} message={}", result.status, result.message);

    Ok(())
}
```

## 依赖的 crates

- `az-desktop-plugin` — 桌面插件框架 trait 定义
- `az-desktop-plugin-registry` — 插件自动注册（`inventory` 机制）
- `gpui` — GPU 加速的 GUI 框架，用于渲染页面视图
- `handlebars` — 模板引擎，渲染步骤中的 URL/Header/Body 模板变量
- `reqwest` — HTTP 客户端，执行网关步骤中的请求
- `serde` / `serde_json` — 序列化与 JSON 处理
- `serde_json_path` — JSONPath 表达式求值，提取响应中的捕获值
- `tokio` — 异步运行时