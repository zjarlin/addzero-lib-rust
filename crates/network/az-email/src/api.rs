//! SMTP 邮件发送客户端，基于 lettre 封装。
//!
//! # 核心类型
//!
//! - [`EmailConfig`] — SMTP 服务器连接配置，支持 SSL/TLS，密码字段在 `Debug` 输出中自动脱敏。
//! - [`EmailMessage`] — 邮件消息构建器，支持纯文本、HTML、多收件人（to/cc/bcc）及文件附件。
//! - [`SmtpEmailSender`] — 实现 [`EmailSender`] trait 的 SMTP 发送器。
//! - [`EmailSenderFactory`] — 依赖注入边界，按 [`EmailSenderConfig`] 构造 boxed sender。
//!
//! # 关键功能
//!
//! - **Builder 模式**：`EmailConfig::builder(...)` 和 `EmailMessage::builder()` 提供链式配置。
//! - **全局默认发送器**：通过 `set_default_sender()` / `clear_default_sender()` 管理进程级默认发送器，
//!   随后可用模块级 `send()` 便捷函数发送邮件。
//! - **快捷函数**：`send_text()` 和 `send_html()` 封装"构建 + 发送"一步到位。
//! - **附件支持**：自动根据文件路径推断 MIME 类型。
//!
//! # 快速开始
//!
//! ```rust,no_run
//! use az_email::api::{EmailConfig, EmailMessage, send_with_config};
//!
//! # fn main() -> anyhow::Result<()> {
//! let config = EmailConfig::builder("smtp.example.com", "user", "pass")
//!     .port(587)
//!     .build()?;
//!
//! let message = EmailMessage::builder()
//!     .from("sender@example.com")
//!     .to("receiver@example.com")
//!     .subject("测试邮件")
//!     .text("这是一封测试邮件。")
//!     .build()?;
//!
//! send_with_config(&config, &message)?;
//! # Ok(())
//! # }
//! ```
use anyhow::{Context, Result, bail};
use az_derive_aliases::{
    apply, from_plain_eq, impl_enum_kind, plain_clone_debug, plain_default_copy_eq,
    plain_default_eq, plain_eq_redacted, serde_code_enum,
};
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

/// SMTP 连接配置。
///
/// `password` 在 `Debug` 输出中会被脱敏；`enable_ssl` 表示 SMTPS wrapper TLS，`enable_tls` 表示普通 SMTP 上要求 TLS。
#[apply(plain_eq_redacted)]
pub struct EmailConfig {
    /// SMTP 服务器主机名。
    pub host: String,
    /// SMTP 服务器端口，默认 587。
    pub port: u16,
    /// SMTP 登录用户名。
    pub username: String,
    /// SMTP 登录密码，调试输出会脱敏。
    #[debug(skip)]
    pub password: String,
    /// 协议名；`smtps` 会触发 wrapper TLS。
    pub protocol: String,
    /// 是否使用 SMTPS wrapper TLS。
    pub enable_ssl: bool,
    /// 是否在普通 SMTP 连接上要求 TLS。
    pub enable_tls: bool,
}

impl EmailConfig {
    /// 创建 SMTP 配置构建器。
    ///
    /// 默认端口为 587，协议为 `smtp`，默认启用 TLS 且不启用 SMTPS wrapper。
    pub fn builder(
        host: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> EmailConfigBuilder {
        Self {
            host: host.into(),
            port: 587,
            username: username.into(),
            password: password.into(),
            protocol: "smtp".to_owned(),
            enable_ssl: false,
            enable_tls: true,
        }
    }

    /// 校验 SMTP 配置的最小可用字段。
    ///
    /// 该方法只做本地字段校验，不尝试连接 SMTP 服务器。
    pub fn validate(&self) -> Result<()> {
        if self.host.trim().is_empty() {
            bail!("invalid email configuration: host cannot be blank");
        }
        if self.username.trim().is_empty() {
            bail!("invalid email configuration: username cannot be blank");
        }
        if self.password.is_empty() {
            bail!("invalid email configuration: password cannot be blank");
        }
        if self.port == 0 {
            bail!("invalid email configuration: port must be greater than zero");
        }
        Ok(())
    }

    /// 设置 SMTP 端口。
    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    /// 设置协议名。
    ///
    /// 当值为 `smtps` 时，传输层会按 SMTPS wrapper TLS 构建。
    pub fn protocol(mut self, value: impl Into<String>) -> Self {
        self.protocol = value.into();
        self
    }

    /// 设置是否使用 SMTPS wrapper TLS。
    pub fn enable_ssl(mut self, value: bool) -> Self {
        self.enable_ssl = value;
        self
    }

    /// 设置是否要求普通 SMTP 连接使用 TLS。
    pub fn enable_tls(mut self, value: bool) -> Self {
        self.enable_tls = value;
        self
    }

    /// 完成构建并执行本地字段校验。
    pub fn build(self) -> Result<EmailConfig> {
        self.validate()?;
        Ok(self)
    }
}

/// `EmailConfig` 采用自身作为轻量 builder。
pub type EmailConfigBuilder = EmailConfig;

/// 待发送的邮件消息。
///
/// 支持纯文本、HTML、多收件人以及本地文件附件；附件路径会在 `build_message` 阶段读取。
#[apply(plain_default_eq)]
pub struct EmailMessage {
    /// 发件人地址。
    pub from: String,
    /// 主收件人列表。
    pub to: Vec<String>,
    /// 抄送收件人列表。
    pub cc: Vec<String>,
    /// 密送收件人列表。
    pub bcc: Vec<String>,
    /// 邮件主题。
    pub subject: String,
    /// 纯文本正文。
    pub text_content: Option<String>,
    /// HTML 正文。
    pub html_content: Option<String>,
    /// 本地附件文件路径列表。
    pub attachments: Vec<String>,
}

impl EmailMessage {
    /// 创建空邮件消息构建器。
    pub fn builder() -> EmailMessageBuilder {
        Self::default()
    }

    /// 校验邮件消息的最小可发送字段。
    ///
    /// 该方法不解析邮箱地址、不读取附件，只检查发件人、收件人和主题是否存在。
    pub fn validate(&self) -> Result<()> {
        if self.from.trim().is_empty() {
            bail!("invalid email message: from cannot be blank");
        }
        if self.to.is_empty() {
            bail!("invalid email message: at least one recipient is required");
        }
        if self.subject.trim().is_empty() {
            bail!("invalid email message: subject cannot be blank");
        }
        Ok(())
    }

    /// 设置发件人地址。
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = value.into();
        self
    }

    /// 追加一个主收件人。
    pub fn to(mut self, value: impl Into<String>) -> Self {
        self.to.push(value.into());
        self
    }

    /// 追加一个抄送收件人。
    pub fn cc(mut self, value: impl Into<String>) -> Self {
        self.cc.push(value.into());
        self
    }

    /// 追加一个密送收件人。
    pub fn bcc(mut self, value: impl Into<String>) -> Self {
        self.bcc.push(value.into());
        self
    }

    /// 设置邮件主题。
    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.subject = value.into();
        self
    }

    /// 设置纯文本正文。
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text_content = Some(value.into());
        self
    }

    /// 设置 HTML 正文。
    pub fn html(mut self, value: impl Into<String>) -> Self {
        self.html_content = Some(value.into());
        self
    }

    /// 追加一个本地附件路径。
    pub fn attachment(mut self, value: impl Into<String>) -> Self {
        self.attachments.push(value.into());
        self
    }

    /// 完成构建并执行本地消息校验。
    pub fn build(self) -> Result<EmailMessage> {
        self.validate()?;
        Ok(self)
    }
}

/// `EmailMessage` 采用自身作为轻量 builder。
pub type EmailMessageBuilder = EmailMessage;

/// 邮件发送器抽象。
///
/// 该 trait 是依赖注入边界，允许测试替身、SMTP 实现或后续其他 provider 共享同一发送入口。
pub trait EmailSender: Send + Sync {
    /// 发送一封已经构建好的邮件消息。
    fn send(&self, message: &EmailMessage) -> Result<()>;
}

/// 可在线程间共享的 boxed 邮件发送器。
pub type BoxEmailSender = Box<dyn EmailSender + Send + Sync>;

/// 邮件发送器类型代码。
///
/// 该枚举的 serde wire value、`code()` 和 `Display` 统一使用 snake_case 约定。
#[apply(serde_code_enum)]
pub enum EmailSenderKind {
    /// SMTP 发送器。
    Smtp,
}

/// 邮件发送器构造配置。
#[apply(from_plain_eq)]
pub enum EmailSenderConfig {
    /// SMTP 发送器配置。
    Smtp(EmailConfig),
}

impl_enum_kind!(EmailSenderConfig => EmailSenderKind, kind {
    Self::Smtp(_) => EmailSenderKind::Smtp,
});

/// 邮件发送器工厂抽象。
///
/// 应用层可以通过该 trait 注入自定义 sender 构造策略，避免业务代码直接依赖 SMTP 实现。
pub trait EmailSenderFactory: Send + Sync {
    /// 根据发送器配置创建 boxed sender。
    fn build_sender(&self, config: EmailSenderConfig) -> Result<BoxEmailSender>;
}

/// 内置发送器工厂。
///
/// 当前只支持 SMTP，后续新增 provider 时应在 `EmailSenderKind` 和 `EmailSenderConfig` 中同步扩展。
#[apply(plain_default_copy_eq)]
pub struct BuiltinEmailSenderFactory;

impl EmailSenderFactory for BuiltinEmailSenderFactory {
    fn build_sender(&self, config: EmailSenderConfig) -> Result<BoxEmailSender> {
        match config {
            EmailSenderConfig::Smtp(config) => Ok(Box::new(SmtpEmailSender::new(config)?)),
        }
    }
}

/// 使用内置工厂创建邮件发送器。
pub fn build_email_sender(config: EmailSenderConfig) -> Result<BoxEmailSender> {
    BuiltinEmailSenderFactory.build_sender(config)
}

/// 基于 `lettre` 的 SMTP 邮件发送器。
///
/// 发送器持有构建好的 `SmtpTransport`，适合作为全局默认 sender 或注入到服务层复用。
#[apply(plain_clone_debug)]
pub struct SmtpEmailSender {
    config: EmailConfig,
    transport: SmtpTransport,
}

impl SmtpEmailSender {
    /// 根据 SMTP 配置构建发送器。
    ///
    /// 构建阶段会执行本地配置校验并初始化 `lettre` transport；实际网络连接通常发生在发送阶段。
    pub fn new(config: EmailConfig) -> Result<Self> {
        config.validate()?;
        let transport = build_transport(&config)?;
        Ok(Self { config, transport })
    }

    /// 返回发送器持有的 SMTP 配置。
    pub fn config(&self) -> &EmailConfig {
        &self.config
    }
}

impl EmailSender for SmtpEmailSender {
    fn send(&self, message: &EmailMessage) -> Result<()> {
        let built = build_message(message)?;
        self.transport.send(&built).context("send email via SMTP")?;
        Ok(())
    }
}

static DEFAULT_SENDER: OnceLock<RwLock<Option<Arc<dyn EmailSender>>>> = OnceLock::new();

/// 注册进程级默认邮件发送器。
///
/// 后续调用 `send` 会使用该 sender；重复调用会覆盖旧 sender。
pub fn set_default_sender(sender: Arc<dyn EmailSender>) {
    let lock = DEFAULT_SENDER.get_or_init(|| RwLock::new(None));
    *match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    } = Some(sender);
}

/// 清除进程级默认邮件发送器。
pub fn clear_default_sender() {
    let lock = DEFAULT_SENDER.get_or_init(|| RwLock::new(None));
    *match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    } = None;
}

/// 使用进程级默认发送器发送邮件。
///
/// 若尚未调用 `set_default_sender`，返回错误。
pub fn send(message: &EmailMessage) -> Result<()> {
    let sender = DEFAULT_SENDER
        .get_or_init(|| RwLock::new(None))
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
        .context("default email sender not configured")?;
    sender.send(message)
}

/// 使用临时 SMTP 配置发送一封邮件。
///
/// 该函数每次调用都会新建 sender，适合低频发送；高频发送建议复用 `SmtpEmailSender`。
pub fn send_with_config(config: &EmailConfig, message: &EmailMessage) -> Result<()> {
    let sender = build_email_sender(config.clone().into())?;
    sender.send(message)
}

/// 构建并发送纯文本邮件。
pub fn send_text(
    config: &EmailConfig,
    from: impl Into<String>,
    to: impl Into<String>,
    subject: impl Into<String>,
    content: impl Into<String>,
) -> Result<()> {
    let message = EmailMessage::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .text(content)
        .build()?;
    send_with_config(config, &message)
}

/// 构建并发送 HTML 邮件。
pub fn send_html(
    config: &EmailConfig,
    from: impl Into<String>,
    to: impl Into<String>,
    subject: impl Into<String>,
    html_content: impl Into<String>,
) -> Result<()> {
    let message = EmailMessage::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .html(html_content)
        .build()?;
    send_with_config(config, &message)
}

/// 将 `EmailMessage` 转换为 `lettre::Message`。
///
/// 该函数会解析邮箱地址、读取附件文件并推断 MIME 类型，但不会连接 SMTP 服务器。
pub fn build_message(message: &EmailMessage) -> Result<Message> {
    message.validate()?;

    let mut builder = Message::builder()
        .from(parse_mailbox(&message.from)?)
        .subject(message.subject.clone());

    for recipient in &message.to {
        builder = builder.to(parse_mailbox(recipient)?);
    }
    for recipient in &message.cc {
        builder = builder.cc(parse_mailbox(recipient)?);
    }
    for recipient in &message.bcc {
        builder = builder.bcc(parse_mailbox(recipient)?);
    }

    let content = build_body(message)?;
    builder.multipart(content).context("build email message")
}

fn parse_mailbox(value: &str) -> Result<Mailbox> {
    value
        .parse::<Mailbox>()
        .with_context(|| format!("invalid email address `{value}`"))
}

fn build_transport(config: &EmailConfig) -> Result<SmtpTransport> {
    let credentials = Credentials::new(config.username.clone(), config.password.clone());
    let mut builder = SmtpTransport::builder_dangerous(&config.host)
        .port(config.port)
        .credentials(credentials);
    let tls_parameters = TlsParameters::new(config.host.clone())
        .with_context(|| format!("build TLS parameters for SMTP host `{}`", config.host))?;

    builder = if config.enable_ssl || config.protocol.eq_ignore_ascii_case("smtps") {
        builder.tls(Tls::Wrapper(tls_parameters))
    } else if config.enable_tls {
        builder.tls(Tls::Required(tls_parameters))
    } else {
        builder.tls(Tls::None)
    };

    Ok(builder.build())
}

fn build_body(message: &EmailMessage) -> Result<MultiPart> {
    let body = match (&message.text_content, &message.html_content) {
        (Some(text), Some(html)) => MultiPart::alternative()
            .singlepart(SinglePart::plain(text.clone()))
            .singlepart(SinglePart::html(html.clone())),
        (Some(text), None) => MultiPart::mixed().singlepart(SinglePart::plain(text.clone())),
        (None, Some(html)) => MultiPart::mixed().singlepart(SinglePart::html(html.clone())),
        (None, None) => MultiPart::mixed().singlepart(SinglePart::plain(String::new())),
    };

    if message.attachments.is_empty() {
        return Ok(body);
    }

    let mut multipart = MultiPart::mixed().multipart(body);
    for attachment in &message.attachments {
        multipart = multipart.singlepart(build_attachment(attachment)?);
    }
    Ok(multipart)
}

fn build_attachment(path: &str) -> Result<SinglePart> {
    let bytes = fs::read(path).with_context(|| format!("failed to read attachment `{path}`"))?;
    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_owned();
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let content_type = ContentType::parse(mime.essence_str())
        .with_context(|| format!("failed to parse content type `{}`", mime.essence_str()))?;

    Ok(Attachment::new(filename).body(bytes, content_type))
}
