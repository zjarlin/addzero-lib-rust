# az-curl

将 curl 命令字符串解析为结构化的 HTTP 请求表示，并支持直接执行。

## 文档策略

`README.md` 是本 crate 的公开文档单一真理源。`src/lib.rs` 通过 `#![doc = include_str!("../README.md")]` 将这份 README 直接引入 rustdoc，避免 README、crate 注释和发布文档三处内容漂移。

## 功能

- 使用 `parse_curl` 解析 curl 命令行字符串，提取方法、URL、请求头、请求体、表单参数等
- 支持常见的 curl 标志（`-X`、`-H`、`-d`、`-b`、`-F`、`-u`、`--data-raw` 等）
- 自动推断 Content-Type（JSON / multipart 表单）
- 提取 URL 中的路径参数和查询参数
- 使用 `execute_curl` 同步执行 curl 命令
- 响应同时保留原始 `body: Vec<u8>` 和便于调试器查看的 `text: Option<String>`
- 公开返回值使用 `error::CurlResult<T>`，具体错误来源保留为结构化 `error::CurlError`

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
use az_curl::parse::parse_curl;

# fn main() -> az_curl::error::CurlResult<()> {
let parsed = parse_curl(r#"curl -X POST -H "Content-Type: application/json" -d '{"key":"value"}' https://api.example.com/data"#)?;
assert_eq!(parsed.method.as_str(), "POST");
assert_eq!(parsed.url, "https://api.example.com/data");
# Ok(())
# }
```

```rust,no_run
use az_curl::execute::execute_curl;

# fn main() -> az_curl::error::CurlResult<()> {
let response = execute_curl(r#"curl -H "Accept: application/json" https://api.example.com"#)?;
println!("{}", response.text()?);

if let Some(text) = response.text.as_deref() {
    println!("debug-visible response body: {text}");
}
# Ok(())
# }
```

## 依赖的 crates

- `automod` - 自动模块声明
- `base64` - Basic 认证编码
- `regex` - URL 路径/查询参数提取及行续接符处理
- `reqwest` - HTTP 客户端（blocking 模式执行请求）
- `shlex` - POSIX shell 词法分析，用于分词 curl 命令
- `thiserror` - 错误类型派生
