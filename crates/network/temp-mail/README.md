# az-temp-mail

临时邮箱 API 客户端，支持自部署的 Cloudflare Workers 临时邮箱服务和 mail.tm 托管服务。

## 功能

- **多后端支持**：通过 `TempMailProvider` trait 统一抽象，内置 Cloudflare Workers、mail.tm、Emailnator 三种实现
- **依赖注入边界**：通过 `TempMailProviderFactory` 和 `TempMailProviderConfig` 构造 boxed provider，方便上层按配置注入
- **地址管理**：创建临时邮箱地址、密码登录、凭据验证、修改密码、删除地址
- **收件箱操作**：列表/详情查看原始邮件和解析后邮件、删除单封邮件、清空收件箱
- **发信功能**：申请发信权限后通过 `/api/send_mail` 发送邮件
- **高级配置**：`CloudflareTempMailContext` 支持自定义认证密钥、地址名称、域名等上下文参数
- **分页查询**：`PageRequest` 统一分页参数，`ListResponse` 统一列表响应格式

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-temp-mail = { path = "../temp-mail" } # workspace 内部引用
# 或发布后：
# az-temp-mail = "0.1"                       # crates.io 引用
```

## 用法

### 使用 Cloudflare Workers 后端

```rust
use az_temp_mail::client::create_temp_mail_api;
use az_temp_mail::model::{NewAddressRequest, PageRequest};

let api = create_temp_mail_api("https://mail.example.com")?;
let address = api.new_address(&NewAddressRequest::new("demo", "example.com"))?;
let inbox = api.list_parsed_mails(&address.jwt, PageRequest::default())?;
println!("{} has {} messages", address.address, inbox.count);
```

### 使用上下文配置

```rust
use az_temp_mail::cloudflare::CloudflareTempMailContext;
use az_temp_mail::model::PageRequest;

let context = CloudflareTempMailContext {
    base_url: "https://mail.example.com".to_owned(),
    custom_auth: Some("admin-secret".to_owned()),
    address_name: Some("demo".to_owned()),
    address_domain: Some("example.com".to_owned()),
    cf_token: None,
    enable_random_subdomain: None,
};
let api = context.create_api()?;
let address = api.new_address(&context.new_address_request())?;
let inbox = api.list_parsed_mails(&address.jwt, PageRequest::default())?;
println!("{} has {} messages", address.address, inbox.count);
```

### 使用 mail.tm 后端

```rust
use az_temp_mail::temp_mail::TempMail;

let api = TempMail::mail_tm()?;
// api 实现了 TempMailProvider trait，可统一操作收件箱
```

### 使用 provider factory

```rust
use az_temp_mail::config::ApiConfig;
use az_temp_mail::model::TempMailProviderKind;
use az_temp_mail::provider::{TempMailProviderConfig, build_temp_mail_provider};

let config = TempMailProviderConfig::MailTm(ApiConfig::builder("https://api.mail.tm").build()?);
assert_eq!(config.kind(), TempMailProviderKind::MailTm);

let provider = build_temp_mail_provider(config)?;
assert_eq!(provider.provider_kind(), TempMailProviderKind::MailTm);

let provider = az_temp_mail::temp_mail::TempMail::provider(TempMailProviderConfig::Emailnator(
    ApiConfig::builder("https://www.emailnator.com").build()?,
))?;
assert_eq!(provider.provider_kind(), TempMailProviderKind::Emailnator);
```

## 依赖的 crates

- `reqwest` - HTTP 客户端，用于调用临时邮箱 API
- `serde` / `serde_json` - JSON 序列化/反序列化
- `sha2` - SHA-256 哈希（用于密码登录时的密码散列）
- `anyhow` - 错误返回与上下文
- `strum` - 稳定 provider code 和字符串转换
