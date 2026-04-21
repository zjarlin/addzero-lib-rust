use addzero_email::*;
use std::fs;
use std::sync::{Arc, Mutex};
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
    let formatted = String::from_utf8(built.formatted()).expect("formatted email should be utf8");

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
