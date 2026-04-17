use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerComposeError {
    #[error("invalid docker run command")]
    InvalidCommandLine,
    #[error("docker image was not found in command")]
    MissingImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerRunCommand {
    pub image: String,
    pub name: Option<String>,
    pub ports: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub volumes: Vec<String>,
    pub network: Option<String>,
    pub restart: Option<String>,
    pub other_options: BTreeMap<String, String>,
}

pub struct DockerComposeConverter;

impl DockerComposeConverter {
    pub fn convert_to_docker_compose(
        docker_run_command: impl AsRef<str>,
    ) -> Result<String, DockerComposeError> {
        DockerRunCommand::parse(docker_run_command).map(|command| command.to_docker_compose_yml())
    }
}

impl DockerRunCommand {
    pub fn parse(command: impl AsRef<str>) -> Result<Self, DockerComposeError> {
        let args = shlex::split(command.as_ref()).ok_or(DockerComposeError::InvalidCommandLine)?;
        let mut args = args.into_iter().peekable();

        let mut image = None;
        let mut name = None;
        let mut ports = Vec::new();
        let mut environment = BTreeMap::new();
        let mut volumes = Vec::new();
        let mut network = None;
        let mut restart = None;
        let mut other_options = BTreeMap::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "docker" | "run" => continue,
                "--name" => name = args.next(),
                "-p" | "--publish" => {
                    if let Some(port) = args.next() {
                        ports.push(port);
                    }
                }
                "-e" | "--env" => {
                    if let Some(env) = args.next() {
                        insert_env(&mut environment, &env);
                    }
                }
                "-v" | "--volume" => {
                    if let Some(volume) = args.next() {
                        volumes.push(volume);
                    }
                }
                "--network" => network = args.next(),
                "--restart" => restart = args.next(),
                _ if arg.starts_with("--name=") => name = Some(arg[7..].to_owned()),
                _ if arg.starts_with("--network=") => network = Some(arg[10..].to_owned()),
                _ if arg.starts_with("--restart=") => restart = Some(arg[10..].to_owned()),
                _ if arg.starts_with("--publish=") => ports.push(arg[10..].to_owned()),
                _ if arg.starts_with("-p") && arg.len() > 2 => ports.push(arg[2..].to_owned()),
                _ if arg.starts_with("--env=") => insert_env(&mut environment, &arg[6..]),
                _ if arg.starts_with("-e") && arg.len() > 2 => {
                    insert_env(&mut environment, &arg[2..]);
                }
                _ if arg.starts_with("--volume=") => volumes.push(arg[9..].to_owned()),
                _ if arg.starts_with("-v") && arg.len() > 2 => volumes.push(arg[2..].to_owned()),
                _ if !arg.starts_with('-') && image.is_none() => image = Some(arg),
                _ if arg.starts_with("--") && arg.contains('=') => {
                    let (key, value) = arg[2..].split_once('=').expect("contains `=`");
                    other_options.insert(key.to_owned(), value.to_owned());
                }
                _ if arg.starts_with("--") => {
                    let key = arg.trim_start_matches("--").to_owned();
                    let value = args
                        .next_if(|value| !value.starts_with('-'))
                        .unwrap_or_default();
                    other_options.insert(key, value);
                }
                _ => {}
            }
        }

        Ok(Self {
            image: image.ok_or(DockerComposeError::MissingImage)?,
            name,
            ports,
            environment,
            volumes,
            network,
            restart,
            other_options,
        })
    }

    pub fn service_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            self.image
                .rsplit('/')
                .next()
                .unwrap_or(self.image.as_str())
                .split(':')
                .next()
                .unwrap_or("app")
                .to_owned()
        })
    }

    pub fn to_docker_compose_yml(&self) -> String {
        let mut yaml = String::new();
        yaml.push_str("version: '3.8'\n");
        yaml.push_str("services:\n");
        yaml.push_str(&format!("  {}:\n", self.service_name()));
        yaml.push_str(&format!("    image: {}\n", self.image));

        if let Some(name) = &self.name {
            yaml.push_str(&format!("    container_name: {}\n", name));
        }
        if !self.ports.is_empty() {
            yaml.push_str("    ports:\n");
            for port in &self.ports {
                yaml.push_str(&format!("      - \"{}\"\n", port));
            }
        }
        if !self.environment.is_empty() {
            yaml.push_str("    environment:\n");
            for (key, value) in &self.environment {
                let escaped = value.replace('\"', "\\\"");
                yaml.push_str(&format!("      {}: \"{}\"\n", key, escaped));
            }
        }
        if !self.volumes.is_empty() {
            yaml.push_str("    volumes:\n");
            for volume in &self.volumes {
                yaml.push_str(&format!("      - \"{}\"\n", volume));
            }
        }
        if let Some(network) = &self.network {
            yaml.push_str("    networks:\n");
            yaml.push_str(&format!("      - {}\n", network));
        }
        if let Some(restart) = &self.restart {
            yaml.push_str(&format!("    restart: {}\n", restart));
        }

        yaml
    }
}

fn insert_env(environment: &mut BTreeMap<String, String>, pair: &str) {
    if let Some((key, value)) = pair.split_once('=') {
        environment.insert(key.to_owned(), value.to_owned());
    } else {
        environment.insert(pair.to_owned(), String::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
