# az-aio-client

AIO Desktop 后端 HTTP 客户端，提供对 Shell 组件注册表和桌面状态 API 的同步调用封装。

## 功能

- 类型安全的 AIO Desktop REST API 客户端，基于 `reqwest::blocking`
- 可选的桌面会话令牌认证（`x-aio-desktop-token` header）
- 完整的 Shell 组件管理接口：列表、获取、插入/更新（upsert）、局部修补（patch）、移除、配置保存、构建
- 桌面后端状态查询（`/api/desktop/status`）
- 统一的 `AioClientError` 错误类型，区分传输层错误与 HTTP 状态错误

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-client = { path = "../az-aio-client" }        # workspace 内部引用
# 或发布后：
# az-aio-client = "0.1"                               # crates.io 引用
```

## 用法

```rust
use az_aio_client::AioClient;

// 创建客户端（无需认证令牌）
let client = AioClient::new("http://localhost:9120");

// 或携带桌面会话令牌
let client = AioClient::with_desktop_token(
    "http://localhost:9120",
    "your-desktop-session-token",
);

// 查询桌面后端状态
let status = client.desktop_status()?;
println!("Desktop OK: {}", status.ok);

// 列出所有 Shell 组件
let registry = client.list_shell_components()?;
println!("共 {} 个组件", registry.components.len());

// 获取单个组件
if let Some(component) = client.get_shell_component("my-alias")? {
    println!("组件预览: {}", component.preview);
}
```

## 依赖的 crates

- `az-config-center-contract` — Shell 组件的共享数据类型
- `reqwest` — HTTP 客户端（阻塞模式）
- `serde` / `serde_json` — JSON 序列化与反序列化
- `thiserror` — 错误类型派生
- `urlencoding` — URL 路径编码（组件名称可能含特殊字符）