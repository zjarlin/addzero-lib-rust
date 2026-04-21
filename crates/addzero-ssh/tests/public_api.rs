use addzero_ssh::*;
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

    match error {
        SshError::InvalidConfig(message) => {
            assert!(message.contains("password or private_key_path"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
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
    match error {
        SshError::CommandFailed { exit_code, stderr } => {
            assert_eq!(exit_code, 2);
            assert_eq!(stderr, "boom");
        }
        other => panic!("unexpected error: {other:?}"),
    }
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
