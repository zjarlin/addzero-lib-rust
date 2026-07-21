use az_ssh::client::{SshConfig, SshExecutionResult};
use std::path::{Path, PathBuf};

#[test]
fn config_builder_matches_jvm_defaults() {
    let config = SshConfig::builder("example.com", "root")
        .password("secret")
        .build()
        .expect("config should build");

    assert_eq!(config.host, "example.com");
    assert_eq!(config.port, 22);
    assert_eq!(config.username, "root");
    assert_eq!(config.password.as_deref(), Some("secret"));
    assert_eq!(config.private_key_path, None);
    assert_eq!(config.connect_timeout_ms, 30_000);
    assert_eq!(config.read_timeout_ms, 60_000);
}

#[test]
fn config_requires_password_or_private_key() {
    let error = SshConfig::builder("example.com", "root")
        .build()
        .expect_err("config should require an auth method");

    assert!(error.to_string().contains("password or private_key_path"));
}

#[test]
fn execution_result_reports_success_and_failure() {
    let success = SshExecutionResult {
        exit_code: 0,
        stdout: "ok".to_owned(),
        stderr: String::new(),
    };
    assert!(success.is_success());
    assert_eq!(
        success
            .get_output_or_throw()
            .expect("success output should return"),
        "ok"
    );

    let failure = SshExecutionResult {
        exit_code: 2,
        stdout: String::new(),
        stderr: "boom".to_owned(),
    };
    let error = failure
        .get_output_or_throw()
        .expect_err("failure should return an error");
    let message = error.to_string();
    assert!(message.contains("exit code 2"));
    assert!(message.contains("boom"));
}

#[test]
fn config_builder_accepts_private_key_authentication() {
    let config = SshConfig::builder("example.com", "root")
        .private_key_path(Path::new("~/demo.txt").display().to_string())
        .build()
        .expect("config should build");

    assert_eq!(config.private_key_path, Some(String::from("~/demo.txt")));
    assert_eq!(config.password, None);
    assert_eq!(config.port, 22);
    assert_eq!(
        PathBuf::from(config.private_key_path.unwrap()),
        PathBuf::from("~/demo.txt")
    );
}

#[test]
fn ssh_config_debug_does_not_leak_credentials() {
    let config = SshConfig::builder("example.com", "alice")
        .password("secret-password")
        .private_key_path("/tmp/id_rsa")
        .private_key_passphrase("ssh-phrase-secret")
        .build()
        .expect("ssh config should build");

    let output = format!("{config:?}");

    assert!(output.contains("example.com"));
    assert!(!output.contains("secret-password"));
    assert!(!output.contains("/tmp/id_rsa"));
    assert!(!output.contains("ssh-phrase-secret"));
}

#[test]
fn ssh_config_builder_debug_masks_credentials_when_present() {
    let config = SshConfig::builder("example.com", "root")
        .password("super-secret")
        .private_key_passphrase("key-passphrase");

    let debug = format!("{config:?}");

    assert!(debug.contains("example.com"));
    assert!(!debug.contains("super-secret"));
    assert!(!debug.contains("key-passphrase"));
}
