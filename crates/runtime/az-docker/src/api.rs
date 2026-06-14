//! Docker 命令行转 Compose 配置转换工具。
//!
//! 本模块承载 crate 的公共 API；crate root 只负责 `automod` 模块收集。
//!
//! 将 `docker run` 命令行字符串解析为结构化的 [`DockerRunCommand`]，
//! 并可生成对应的 Docker Compose v3.8 YAML 配置。
//!
//! # 核心类型
//!
//! - [`DockerRunCommand`] — 解析后的 Docker Run 命令结构，包含镜像、端口、环境变量等字段。
//! - [`DockerComposeConverter`] — 转换器，提供一行式 API 将命令字符串直接转为 YAML。
//!
//! # 主要功能
//!
//! - 支持多种参数格式（`-p 8080:80`、`-p8080:80`、`--publish=8080:80`）
//! - 通过 `shlex` 正确处理 shell 引号与转义
//! - 未知的 `--key=value` 选项会被归入 `other_options` 字段

use anyhow::bail;
use az_derive_aliases::{apply, plain_eq};
use std::collections::BTreeMap;
use std::str::FromStr;

/// 解析后的 `docker run` 命令结构。
///
/// 当前模型只保留生成 Compose 所需的稳定字段；未知长选项会进入 [`Self::other_options`]，
/// 避免解析阶段直接丢失信息。
#[apply(plain_eq)]
pub struct DockerRunCommand {
    /// 容器镜像名。
    pub image: String,
    /// `--name` 指定的容器名。
    pub name: Option<String>,
    /// 端口映射，保留 `host:container` 原始格式。
    pub ports: Vec<String>,
    /// 环境变量键值对。
    pub environment: BTreeMap<String, String>,
    /// 卷挂载声明。
    pub volumes: Vec<String>,
    /// Docker 网络名称。
    pub network: Option<String>,
    /// 容器重启策略。
    pub restart: Option<String>,
    /// 当前转换器尚未建模的长选项。
    pub other_options: BTreeMap<String, String>,
}

/// `docker run` 命令到 Docker Compose YAML 的一行式转换入口。
pub struct DockerComposeConverter;

impl DockerComposeConverter {
    /// 将 `docker run` 命令字符串转换为 Compose v3.8 YAML。
    pub fn convert_to_docker_compose(
        docker_run_command: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        DockerRunCommand::parse(docker_run_command).map(|command| command.to_docker_compose_yml())
    }
}

impl FromStr for DockerRunCommand {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl DockerRunCommand {
    /// 按 shell 引号规则解析 `docker run` 命令。
    ///
    /// 支持常见短参数、长参数和 `--key=value` 形式；第一个非选项参数会被视为镜像名。
    pub fn parse(command: impl AsRef<str>) -> anyhow::Result<Self> {
        let Some(args) = shlex::split(command.as_ref()) else {
            bail!("invalid docker run command");
        };
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
            image: image.ok_or_else(|| anyhow::anyhow!("docker image was not found in command"))?,
            name,
            ports,
            environment,
            volumes,
            network,
            restart,
            other_options,
        })
    }

    /// 返回 Compose service 名称。
    ///
    /// 优先使用 `--name`；未指定时从镜像名去掉 registry 路径和 tag 后推导。
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

    /// 生成最小 Docker Compose v3.8 YAML 文本。
    ///
    /// 该函数只做字符串渲染，不执行 Docker，也不创建文件。
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
