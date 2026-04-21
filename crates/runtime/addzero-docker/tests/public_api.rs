use addzero_docker::*;

#[test]
fn parse_docker_run_command_supports_common_flags() {
    let command = DockerRunCommand::parse(
        r#"docker run --name web -p 8080:80 -e APP_ENV=prod -v /tmp/data:/data --network app-net --restart=always nginx:1.27"#,
    )
    .expect("command should parse");

    assert_eq!(command.image, "nginx:1.27");
    assert_eq!(command.name.as_deref(), Some("web"));
    assert_eq!(command.ports, vec!["8080:80".to_owned()]);
    assert_eq!(
        command.environment.get("APP_ENV").map(String::as_str),
        Some("prod")
    );
    assert_eq!(command.volumes, vec!["/tmp/data:/data".to_owned()]);
    assert_eq!(command.network.as_deref(), Some("app-net"));
    assert_eq!(command.restart.as_deref(), Some("always"));
}

#[test]
fn parse_docker_run_command_supports_inline_flags_and_other_options() {
    let command = DockerRunCommand::parse(
        r#"docker run -p127.0.0.1:6379:6379 --env=ALLOW_EMPTY=yes --volume=/data:/var/lib/redis --cpus=2 redis:7"#,
    )
    .expect("command should parse");

    assert_eq!(command.service_name(), "redis");
    assert_eq!(command.ports, vec!["127.0.0.1:6379:6379".to_owned()]);
    assert_eq!(
        command.environment.get("ALLOW_EMPTY").map(String::as_str),
        Some("yes")
    );
    assert_eq!(command.volumes, vec!["/data:/var/lib/redis".to_owned()]);
    assert_eq!(
        command.other_options.get("cpus").map(String::as_str),
        Some("2")
    );
}

#[test]
fn docker_compose_output_contains_expected_sections() {
    let yaml = DockerComposeConverter::convert_to_docker_compose(
        r#"docker run --name app -p 3000:3000 -e TOKEN="abc def" node:20"#,
    )
    .expect("yaml should be generated");

    assert!(yaml.contains("version: '3.8'"));
    assert!(yaml.contains("container_name: app"));
    assert!(yaml.contains("- \"3000:3000\""));
    assert!(yaml.contains("TOKEN: \"abc def\""));
}

#[test]
fn missing_image_returns_error() {
    let error =
        DockerRunCommand::parse("docker run --name app").expect_err("image should be required");

    assert!(matches!(error, DockerComposeError::MissingImage));
}
