use az_email::sender::EmailConfig;

#[test]
fn email_config_debug_does_not_leak_password() {
    let config = EmailConfig::builder("smtp.example.com", "mailer", "top-secret")
        .build()
        .expect("email config should build");

    let output = format!("{config:?}");
    assert!(output.contains("smtp.example.com"));
    assert!(!output.contains("top-secret"));
}
