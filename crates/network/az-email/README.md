# az-email

基于 lettre 封装的 SMTP 邮件发送客户端，支持纯文本、HTML、附件及全局默认发送器。

## 功能

- SMTP 连接配置（SSL / TLS），密码字段 Debug 自动脱敏
- Builder 模式构建 `EmailConfig` 和 `EmailMessage`
- 支持 to、cc、bcc 多收件人
- 纯文本 / HTML 内容及文件附件（自动 MIME 推断）
- 全局默认发送器管理（`set_default_sender` / `send`）
- 快捷发送函数：`send_text()`、`send_html()`

## 安装

在 `Cargo.toml` 中添加：

\`\`\`toml
[dependencies]
az-email = { path = "../az-email" }        # workspace 内部引用
# 或发布后：
# az-email = "0.1"                         # crates.io 引用
\`\`\`

## 用法

\`\`\`rust,no_run
use az_email::{EmailConfig, EmailMessage, send_with_config};

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
use az_email::{EmailConfig, SmtpEmailSender, EmailMessage, set_default_sender, send};
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

## 依赖的 crates

- `lettre` — SMTP 协议实现
- `mime_guess` — 文件 MIME 类型推断（用于附件）
- `thiserror` — 错误类型派生
