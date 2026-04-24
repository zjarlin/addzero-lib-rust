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
                "docker" | "run" => {}
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
                    if let Some((key, value)) = arg[2..].split_once('=') {
                        other_options.insert(key.to_owned(), value.to_owned());
                    }
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

    #[must_use]
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

    #[must_use]
    pub fn to_docker_compose_yml(&self) -> String {
        let mut yaml = String::new();
        yaml.push_str("version: '3.8'\n");
        yaml.push_str("services:\n");
        yaml.push_str("  ");
        yaml.push_str(&self.service_name());
        yaml.push_str(":\n");
        yaml.push_str("    image: ");
        yaml.push_str(&self.image);
        yaml.push('\n');

        if let Some(name) = &self.name {
            yaml.push_str("    container_name: ");
            yaml.push_str(name);
            yaml.push('\n');
        }
        if !self.ports.is_empty() {
            yaml.push_str("    ports:\n");
            for port in &self.ports {
                yaml.push_str("      - \"");
                yaml.push_str(port);
                yaml.push_str("\"\n");
            }
        }
        if !self.environment.is_empty() {
            yaml.push_str("    environment:\n");
            for (key, value) in &self.environment {
                let escaped = value.replace('\"', "\\\"");
                yaml.push_str("      ");
                yaml.push_str(key);
                yaml.push_str(": \"");
                yaml.push_str(&escaped);
                yaml.push_str("\"\n");
            }
        }
        if !self.volumes.is_empty() {
            yaml.push_str("    volumes:\n");
            for volume in &self.volumes {
                yaml.push_str("      - \"");
                yaml.push_str(volume);
                yaml.push_str("\"\n");
            }
        }
        if let Some(network) = &self.network {
            yaml.push_str("    networks:\n");
            yaml.push_str("      - ");
            yaml.push_str(network);
            yaml.push('\n');
        }
        if let Some(restart) = &self.restart {
            yaml.push_str("    restart: ");
            yaml.push_str(restart);
            yaml.push('\n');
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
