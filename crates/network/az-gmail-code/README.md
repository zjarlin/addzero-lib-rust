# az-gmail-code

通过 Gmail API 从调用者拥有的邮箱中读取验证码。

本 crate 不是绕过验证的第三方"接码"服务。它需要一个调用者控制的 Gmail OAuth 访问令牌，通过 Gmail API 搜索邮件，并从邮件正文中提取短数字验证码。

## 坐标

Workspace crate：

```toml
az-gmail-code = { workspace = true }
```

## 示例

```rust,no_run
use az_gmail_code::{GmailCodeClient, GmailCodeQuery};

# fn run() -> az_gmail_code::GmailCodeResult<()> {
let client = GmailCodeClient::new("ya29.access-token")?;
let code = client.find_latest_code(
    GmailCodeQuery::new()
        .from("security@example.com")
        .subject("verification")
        .newer_than("10m")
        .unread(true),
)?;

if let Some(code) = code {
    println!("{} from {}", code.code, code.message_id);
}
# Ok(())
# }
```

## 运行时约束

- 需要目标邮箱的 Gmail API OAuth 访问权限。
- 使用 Gmail 的 `users.messages.list` 和 `users.messages.get` 端点。
- OAuth 作用域应尽可能窄，通常使用 `https://www.googleapis.com/auth/gmail.readonly`。

## 获取 Gmail OAuth 访问令牌

OAuth 在 `az-oauth2` 中独立实现，因为它不局限于 Gmail。

```rust,no_run
use az_oauth2::{
    AuthorizationCodeOptions, GoogleOAuth2, OAuth2Client,
};

# fn run() -> Result<(), Box<dyn std::error::Error>> {
let oauth = OAuth2Client::new(
    GoogleOAuth2::installed_app("google-client-id")
        .scope(GoogleOAuth2::GMAIL_READONLY_SCOPE)
        .build()?,
)?;

let session = oauth.begin_loopback_authorization(
    AuthorizationCodeOptions::new()
        .access_type("offline")
        .prompt("consent"),
)?;

println!("Open: {}", session.authorization_url);
let pkce = session.pkce.clone();
let callback = session.wait_for_callback(std::time::Duration::from_secs(300))?;
let token = oauth.exchange_authorization_code(
    &callback.code,
    &callback.redirect_uri,
    Some(&pkce),
)?;

let gmail = az_gmail_code::GmailCodeClient::new(token.require_access_token()?)?;
# Ok(())
# }
```

## 依赖的 crates

- `az-oauth2` — OAuth2 授权码和设备流程
- `base64` — 邮件正文 Base64 解码
- `regex` — 从邮件中提取验证码正则
- `reqwest` — 调用 Gmail REST API
- `serde` / `serde_json` — Gmail API JSON 响应反序列化
- `thiserror` — 错误类型派生
- `urlencoding` — Gmail 查询参数编码
