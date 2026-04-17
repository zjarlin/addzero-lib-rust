use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("invalid email configuration: {0}")]
    InvalidConfig(String),
    #[error("invalid email message: {0}")]
    InvalidMessage(String),
    #[error("invalid email address: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("failed to build email message: {0}")]
    Build(#[from] lettre::error::Error),
    #[error("failed to parse content type `{value}`: {source}")]
    ContentType {
        value: String,
        #[source]
        source: lettre::message::header::ContentTypeErr,
    },
    #[error("smtp transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[error("failed to read attachment `{path}`: {source}")]
    AttachmentIo {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("default email sender not configured")]
    MissingDefaultSender,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub protocol: String,
    pub enable_ssl: bool,
    pub enable_tls: bool,
}

impl EmailConfig {
    pub fn builder(
        host: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> EmailConfigBuilder {
        EmailConfigBuilder {
            host: host.into(),
            port: 587,
            username: username.into(),
            password: password.into(),
            protocol: "smtp".to_owned(),
            enable_ssl: false,
            enable_tls: true,
        }
    }

    pub fn validate(&self) -> Result<(), EmailError> {
        if self.host.trim().is_empty() {
            return Err(EmailError::InvalidConfig("host cannot be blank".to_owned()));
        }
        if self.username.trim().is_empty() {
            return Err(EmailError::InvalidConfig(
                "username cannot be blank".to_owned(),
            ));
        }
        if self.password.is_empty() {
            return Err(EmailError::InvalidConfig(
                "password cannot be blank".to_owned(),
            ));
        }
        if self.port == 0 {
            return Err(EmailError::InvalidConfig(
                "port must be greater than zero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EmailConfigBuilder {
    host: String,
    port: u16,
    username: String,
    password: String,
    protocol: String,
    enable_ssl: bool,
    enable_tls: bool,
}

impl EmailConfigBuilder {
    pub fn port(mut self, value: u16) -> Self {
        self.port = value;
        self
    }

    pub fn protocol(mut self, value: impl Into<String>) -> Self {
        self.protocol = value.into();
        self
    }

    pub fn enable_ssl(mut self, value: bool) -> Self {
        self.enable_ssl = value;
        self
    }

    pub fn enable_tls(mut self, value: bool) -> Self {
        self.enable_tls = value;
        self
    }

    pub fn build(self) -> Result<EmailConfig, EmailError> {
        let config = EmailConfig {
            host: self.host,
            port: self.port,
            username: self.username,
            password: self.password,
            protocol: self.protocol,
            enable_ssl: self.enable_ssl,
            enable_tls: self.enable_tls,
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailMessage {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub text_content: Option<String>,
    pub html_content: Option<String>,
    pub attachments: Vec<String>,
}

impl EmailMessage {
    pub fn builder() -> EmailMessageBuilder {
        EmailMessageBuilder::default()
    }

    pub fn validate(&self) -> Result<(), EmailError> {
        if self.from.trim().is_empty() {
            return Err(EmailError::InvalidMessage(
                "from cannot be blank".to_owned(),
            ));
        }
        if self.to.is_empty() {
            return Err(EmailError::InvalidMessage(
                "at least one recipient is required".to_owned(),
            ));
        }
        if self.subject.trim().is_empty() {
            return Err(EmailError::InvalidMessage(
                "subject cannot be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EmailMessageBuilder {
    from: Option<String>,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: Option<String>,
    text_content: Option<String>,
    html_content: Option<String>,
    attachments: Vec<String>,
}

impl EmailMessageBuilder {
    pub fn from(mut self, value: impl Into<String>) -> Self {
        self.from = Some(value.into());
        self
    }

    pub fn to(mut self, value: impl Into<String>) -> Self {
        self.to.push(value.into());
        self
    }

    pub fn cc(mut self, value: impl Into<String>) -> Self {
        self.cc.push(value.into());
        self
    }

    pub fn bcc(mut self, value: impl Into<String>) -> Self {
        self.bcc.push(value.into());
        self
    }

    pub fn subject(mut self, value: impl Into<String>) -> Self {
        self.subject = Some(value.into());
        self
    }

    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text_content = Some(value.into());
        self
    }

    pub fn html(mut self, value: impl Into<String>) -> Self {
        self.html_content = Some(value.into());
        self
    }

    pub fn attachment(mut self, value: impl Into<String>) -> Self {
        self.attachments.push(value.into());
        self
    }

    pub fn build(self) -> Result<EmailMessage, EmailError> {
        let message = EmailMessage {
            from: self.from.unwrap_or_default(),
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            subject: self.subject.unwrap_or_default(),
            text_content: self.text_content,
            html_content: self.html_content,
            attachments: self.attachments,
        };
        message.validate()?;
        Ok(message)
    }
}

pub trait EmailSender: Send + Sync {
    fn send(&self, message: &EmailMessage) -> Result<(), EmailError>;
}

#[derive(Debug, Clone)]
pub struct SmtpEmailSender {
    config: EmailConfig,
    transport: SmtpTransport,
}

impl SmtpEmailSender {
    pub fn new(config: EmailConfig) -> Result<Self, EmailError> {
        config.validate()?;
        let transport = build_transport(&config)?;
        Ok(Self { config, transport })
    }

    pub fn config(&self) -> &EmailConfig {
        &self.config
    }
}

impl EmailSender for SmtpEmailSender {
    fn send(&self, message: &EmailMessage) -> Result<(), EmailError> {
        let built = build_message(message)?;
        self.transport.send(&built)?;
        Ok(())
    }
}

static DEFAULT_SENDER: OnceLock<RwLock<Option<Arc<dyn EmailSender>>>> = OnceLock::new();

pub fn set_default_sender(sender: Arc<dyn EmailSender>) {
    let lock = DEFAULT_SENDER.get_or_init(|| RwLock::new(None));
    *lock
        .write()
        .expect("email sender lock should not be poisoned") = Some(sender);
}

pub fn clear_default_sender() {
    let lock = DEFAULT_SENDER.get_or_init(|| RwLock::new(None));
    *lock
        .write()
        .expect("email sender lock should not be poisoned") = None;
}

pub fn send(message: &EmailMessage) -> Result<(), EmailError> {
    let sender = DEFAULT_SENDER
        .get_or_init(|| RwLock::new(None))
        .read()
        .expect("email sender lock should not be poisoned")
        .clone()
        .ok_or(EmailError::MissingDefaultSender)?;
    sender.send(message)
}

pub fn send_with_config(config: &EmailConfig, message: &EmailMessage) -> Result<(), EmailError> {
    let sender = SmtpEmailSender::new(config.clone())?;
    sender.send(message)
}

pub fn send_text(
    config: &EmailConfig,
    from: impl Into<String>,
    to: impl Into<String>,
    subject: impl Into<String>,
    content: impl Into<String>,
) -> Result<(), EmailError> {
    let message = EmailMessage::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .text(content)
        .build()?;
    send_with_config(config, &message)
}

pub fn send_html(
    config: &EmailConfig,
    from: impl Into<String>,
    to: impl Into<String>,
    subject: impl Into<String>,
    html_content: impl Into<String>,
) -> Result<(), EmailError> {
    let message = EmailMessage::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .html(html_content)
        .build()?;
    send_with_config(config, &message)
}

pub fn build_message(message: &EmailMessage) -> Result<Message, EmailError> {
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
    builder.multipart(content).map_err(EmailError::Build)
}

fn parse_mailbox(value: &str) -> Result<Mailbox, EmailError> {
    value.parse::<Mailbox>().map_err(EmailError::Address)
}

fn build_transport(config: &EmailConfig) -> Result<SmtpTransport, EmailError> {
    let credentials = Credentials::new(config.username.clone(), config.password.clone());
    let mut builder = SmtpTransport::builder_dangerous(&config.host)
        .port(config.port)
        .credentials(credentials);
    let tls_parameters = TlsParameters::new(config.host.clone())?;

    builder = if config.enable_ssl || config.protocol.eq_ignore_ascii_case("smtps") {
        builder.tls(Tls::Wrapper(tls_parameters))
    } else if config.enable_tls {
        builder.tls(Tls::Required(tls_parameters))
    } else {
        builder.tls(Tls::None)
    };

    Ok(builder.build())
}

fn build_body(message: &EmailMessage) -> Result<MultiPart, EmailError> {
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

fn build_attachment(path: &str) -> Result<SinglePart, EmailError> {
    let bytes = fs::read(path).map_err(|source| EmailError::AttachmentIo {
        path: path.to_owned(),
        source,
    })?;
    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_owned();
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let content_type =
        ContentType::parse(mime.essence_str()).map_err(|source| EmailError::ContentType {
            value: mime.essence_str().to_owned(),
            source,
        })?;

    Ok(Attachment::new(filename).body(bytes, content_type))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    #[derive(Debug, Default)]
    struct RecordingSender {
        messages: Mutex<Vec<EmailMessage>>,
    }

    impl EmailSender for RecordingSender {
        fn send(&self, message: &EmailMessage) -> Result<(), EmailError> {
            self.messages
                .lock()
                .expect("recording sender mutex should not be poisoned")
                .push(message.clone());
            Ok(())
        }
    }

    #[test]
    fn config_builder_applies_jvm_defaults() {
        let config = EmailConfig::builder("smtp.example.com", "user@example.com", "secret")
            .build()
            .expect("config should build");

        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.protocol, "smtp");
        assert!(!config.enable_ssl);
        assert!(config.enable_tls);
    }

    #[test]
    fn message_builder_collects_all_fields() {
        let message = EmailMessage::builder()
            .from("sender@example.com")
            .to("a@example.com")
            .to("b@example.com")
            .cc("cc@example.com")
            .bcc("bcc@example.com")
            .subject("Subject")
            .text("plain")
            .html("<b>html</b>")
            .attachment("/tmp/demo.txt")
            .build()
            .expect("message should build");

        assert_eq!(message.from, "sender@example.com");
        assert_eq!(message.to, vec!["a@example.com", "b@example.com"]);
        assert_eq!(message.cc, vec!["cc@example.com"]);
        assert_eq!(message.bcc, vec!["bcc@example.com"]);
        assert_eq!(message.subject, "Subject");
        assert_eq!(message.text_content.as_deref(), Some("plain"));
        assert_eq!(message.html_content.as_deref(), Some("<b>html</b>"));
        assert_eq!(message.attachments, vec!["/tmp/demo.txt"]);
    }

    #[test]
    fn build_message_creates_multipart_email_with_attachment() {
        let attachment = NamedTempFile::new().expect("tempfile should exist");
        fs::write(attachment.path(), "hello attachment").expect("attachment should write");

        let message = EmailMessage::builder()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Hello")
            .text("text body")
            .html("<p>html body</p>")
            .attachment(attachment.path().display().to_string())
            .build()
            .expect("message should build");

        let built = build_message(&message).expect("lettre message should build");
        let formatted =
            String::from_utf8(built.formatted()).expect("formatted email should be utf8");

        assert!(formatted.contains("Subject: Hello"));
        assert!(formatted.contains("recipient@example.com"));
        assert!(formatted.contains("text body"));
        assert!(formatted.contains("html body"));
        assert!(
            formatted.contains(
                attachment
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("attachment filename should exist")
            )
        );
    }

    #[test]
    fn default_sender_dispatches_to_registered_sender() {
        let sender = Arc::new(RecordingSender::default());
        set_default_sender(sender.clone());

        let message = EmailMessage::builder()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Hello")
            .text("body")
            .build()
            .expect("message should build");

        send(&message).expect("default sender should send");
        clear_default_sender();

        let recorded = sender
            .messages
            .lock()
            .expect("recording sender mutex should not be poisoned");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].subject, "Hello");
    }

    #[test]
    fn smtp_sender_construction_validates_configuration() {
        let config = EmailConfig::builder("smtp.example.com", "user@example.com", "secret")
            .port(465)
            .protocol("smtps")
            .enable_ssl(true)
            .enable_tls(false)
            .build()
            .expect("config should build");

        let sender = SmtpEmailSender::new(config.clone()).expect("smtp sender should build");

        assert_eq!(sender.config(), &config);
    }
}
