# az-oauth2

面向已安装应用、CLI 工具和输入受限设备的、与提供商无关的 OAuth2 辅助工具。

本 crate 只实现可复用的 OAuth2 协议机制：

- PKCE verifier / challenge 生成
- authorization-code URL 构造
- 用于桌面/CLI 应用的本地回环（loopback）重定向监听器
- authorization code 交换
- refresh token 交换
- 设备授权与轮询

各提供商的特有行为放在薄层的配置辅助工具中。这里对 Google 的支持仅限于官方的 endpoint/scope 常量和一个配置构造函数。

## Gmail 只读 Token 示例

```rust,no_run
use az_oauth2::client::OAuth2Client;
use az_oauth2::config::AuthorizationCodeOptions;
use az_oauth2::google::GoogleOAuth2;

# fn run() -> anyhow::Result<()> {
let config = GoogleOAuth2::installed_app("google-client-id")
    .scope(GoogleOAuth2::GMAIL_READONLY_SCOPE)
    .build()?;
let client = OAuth2Client::new(config)?;

let session = client.begin_loopback_authorization(
    AuthorizationCodeOptions::new()
        .access_type("offline")
        .prompt("consent"),
)?;

println!("请打开: {}", session.authorization_url);
let pkce = session.pkce.clone();
let callback = session.wait_for_callback(std::time::Duration::from_secs(300))?;
let token = client.exchange_authorization_code(
    &callback.code,
    &callback.redirect_uri,
    Some(&pkce),
)?;

println!("{}", token.require_access_token()?);
# Ok(())
# }
```

将返回的 access token 与 `az-gmail-code` 配合使用。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-oauth2 = { path = "../az-oauth2" }    # workspace 内部引用
# 或发布后：
# az-oauth2 = "0.1"                      # crates.io 引用
```

## 依赖的 crates

- `base64` — PKCE code_verifier 编码
- `reqwest` — HTTP 客户端（异步）
- `ring` — 安全的随机数生成
- `sha2` — PKCE code_challenge 的哈希计算
- `urlencoding` — URL 编码
