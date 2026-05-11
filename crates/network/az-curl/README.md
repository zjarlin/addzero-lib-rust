# az-curl

将 curl 命令字符串解析为结构化的 HTTP 请求表示，并支持直接执行。

## 功能

- 解析 curl 命令行字符串，提取方法、URL、请求头、请求体、表单参数等
- 支持常见的 curl 标志（`-X`、`-H`、`-d`、`-b`、`-F`、`-u`、`--data-raw` 等）
- 自动推断 Content-Type（JSON / multipart 表单）
- 提取 URL 中的路径参数和查询参数，支持路径参数变异规则
- 通过 `curl!` 宏在编译期解析 curl 命令
- 提供 `CurlBuilder` 用于以 Builder 模式程序化构建请求
- 使用 `reqwest::blocking` 同步执行已解析的 HTTP 请求

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-curl = { path = "../az-curl" }         # workspace 内部引用
# 或发布后：
# az-curl = "0.1"                          # crates.io 引用
```

## 用法

```rust
use az_curl::CurlParser;

// 解析 curl 命令字符串
let parsed = CurlParser::parse(r#"curl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com/data"#)?;
assert_eq!(parsed.method, reqwest::Method::POST);
assert_eq!(parsed.url, "https://api.example.com/data");
```

```rust
use az_curl::ParsedCurl;
use reqwest::Method;

// 使用 Builder 模式构建请求
let req = ParsedCurl::builder("https://api.example.com/users")
    .method("POST")?
    .header("Authorization", "Bearer token123")
    .body(r#"{"name":"test"}"#)
    .build()?;
```

```rust
use az_curl::curl;

// 使用宏在编译期解析
let parsed = curl!(r#"curl https://example.com"#);
```

## 依赖的 crates

- `base64` - Basic 认证编码
- `regex` - URL 路径/查询参数提取及行续接符处理
- `reqwest` - HTTP 客户端（blocking 模式执行请求）
- `serde` / `serde_json` - JSON 载荷序列化
- `shlex` - POSIX shell 词法分析，用于分词 curl 命令
- `thiserror` - 错误类型派生
