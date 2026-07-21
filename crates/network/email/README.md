# az-email

基于 lettre 封装的 SMTP 邮件发送客户端，支持纯文本、HTML、附件及全局默认发送器。

## 功能

- SMTP 连接配置（SSL / TLS），密码字段 Debug 自动脱敏
- Builder 模式构建 `EmailConfig` 和 `EmailMessage`
- 支持 to、cc、bcc 多收件人
- 纯文本 / HTML 内容及文件附件（自动 MIME 推断）
- 全局默认发送器管理（`set_default_sender` / `send`）
- `EmailSenderFactory` 注入边界，可按 `EmailSenderConfig` 构造 boxed sender
- `EmailSenderKind` 提供 snake_case code / serde / strum 枚举约定
- 快捷发送函数：`send_text()`、`send_html()`

## 安装

在 `Cargo.toml` 中添加：

\`\`\`toml
[dependencies]
az-email = { path = "../email" }        # workspace 内部引用
# 或发布后：
# az-email = "0.1"                         # crates.io 引用
\`\`\`

## 用法

\`\`\`rust,no_run
use az_email::sender::{EmailConfig, EmailMessage, send_with_config};

// 构建配置
let config = EmailConfig::builder("smtp.example.com", "user", "pass")
    .port(587)
    .enable_tls(true)
    .build()
    .unwrap();

// 构建邮件
let message = EmailMessage::builder()
    .from("sender@example.com")
    .to("receiver@example.com")
    .subject("问候")
    .text("你好！")
    .build()
    .unwrap();

// 发送
send_with_config(&config, &message).unwrap();
\`\`\`

### 全局默认发送器

\`\`\`rust,no_run
use az_email::sender::{EmailConfig, EmailMessage, SmtpEmailSender, send, set_default_sender};
use std::sync::Arc;

let config = EmailConfig::builder("smtp.example.com", "user", "pass").build().unwrap();
let sender = SmtpEmailSender::new(config).unwrap();
set_default_sender(Arc::new(sender));

let msg = EmailMessage::builder()
    .from("s@example.com")
    .to("r@example.com")
    .subject("hi")
    .text("hello")
    .build()
    .unwrap();

send(&msg).unwrap();
\`\`\`

### 通过 factory 构造 sender

\`\`\`rust,no_run
use az_email::sender::{EmailConfig, EmailSenderConfig, EmailSenderKind, build_email_sender};

let config = EmailConfig::builder("smtp.example.com", "user", "pass")
    .build()
    .unwrap();
let sender_config = EmailSenderConfig::Smtp(config);
assert_eq!(sender_config.kind(), EmailSenderKind::Smtp);
assert_eq!(EmailSenderKind::Smtp.code(), "smtp");

let sender = build_email_sender(sender_config).unwrap();
\`\`\`

## 依赖的 crates

- `lettre` — SMTP 协议实现
- `mime_guess` — 文件 MIME 类型推断（用于附件）
- `anyhow` — 错误链与上下文
- `derive_more` — `EmailSenderConfig` 的轻量 `From` 派生
- `strum` — `EmailSenderKind` 的 code / display / parse / variants 派生
