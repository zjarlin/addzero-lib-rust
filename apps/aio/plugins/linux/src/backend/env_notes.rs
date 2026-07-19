//! 环境搭建笔记脚本目录。
//!
//! AZ Linux 不在插件里重新维护 Docker、换源和 SSH 初始化脚本；客户端只从
//! `/Users/zjarlin/aio/note/环境搭建` 读取既有笔记命令，并把它们组合进引导计划。

use std::{collections::BTreeMap, env, fs, path::Path};

use crate::backend::contract::{
    EnvironmentSetupCatalog, EnvironmentSetupCommand, EnvironmentSetupSourceSummary,
};

const ENV_SETUP_DIR_ENV: &str = "AZ_LINUX_ENV_SETUP_DIR";
const DEFAULT_ENV_SETUP_DIR: &str = "/Users/zjarlin/aio/note/环境搭建";
const SOURCE_DOCKER_MANUAL: &str = "docker环境搭建手册.md";
const SOURCE_MAC_MANUAL: &str = "新mac配置文档.md";

pub fn load_environment_setup_catalog() -> EnvironmentSetupCatalog {
    let source_root = env::var(ENV_SETUP_DIR_ENV).unwrap_or_else(|_| DEFAULT_ENV_SETUP_DIR.to_string());
    load_environment_setup_catalog_from_root(&source_root)
}

pub fn setup_source_summary(catalog: &EnvironmentSetupCatalog) -> EnvironmentSetupSourceSummary {
    EnvironmentSetupSourceSummary {
        source_root: catalog.source_root.clone(),
        source_files: catalog.source_files.clone(),
        command_count: catalog.commands.len(),
        available: catalog.source_files.iter().all(|file| file.exists),
    }
}

pub fn catalog_command<'a>(
    catalog: &'a EnvironmentSetupCatalog,
    id: &str,
) -> Option<&'a EnvironmentSetupCommand> {
    catalog.commands.iter().find(|command| command.id == id)
}

fn load_environment_setup_catalog_from_root(source_root: &str) -> EnvironmentSetupCatalog {
    let mut source_files = Vec::new();
    let mut documents = Vec::new();
    let root = Path::new(source_root);

    for file_name in [SOURCE_DOCKER_MANUAL, SOURCE_MAC_MANUAL] {
        let path = root.join(file_name);
        let path_text = path.to_string_lossy().into_owned();
        match fs::read_to_string(&path) {
            Ok(text) => {
                source_files.push(crate::backend::contract::EnvironmentSetupSourceFile {
                    path: path_text.clone(),
                    exists: true,
                });
                documents.push(MarkdownDocument {
                    source_path: path_text,
                    text,
                });
            }
            Err(_) => {
                source_files.push(crate::backend::contract::EnvironmentSetupSourceFile {
                    path: path_text,
                    exists: false,
                });
            }
        }
    }

    EnvironmentSetupCatalog {
        source_root: source_root.to_string(),
        source_files,
        commands: collect_commands_from_documents(&documents),
    }
}

struct MarkdownDocument {
    source_path: String,
    text: String,
}

fn collect_commands_from_documents(documents: &[MarkdownDocument]) -> Vec<EnvironmentSetupCommand> {
    let mut commands = BTreeMap::new();
    for document in documents {
        for (index, line) in document.text.lines().enumerate() {
            if let Some(command) = command_from_line(&document.source_path, index + 1, line) {
                commands.entry(command.id.clone()).or_insert(command);
            }
        }
    }
    commands.into_values().collect()
}

fn command_from_line(
    source_path: &str,
    source_line: usize,
    raw_line: &str,
) -> Option<EnvironmentSetupCommand> {
    let command = cleanup_markdown_line(raw_line)?;
    if command.contains("LinuxMirrors/raw/main/ChangeMirrors.sh") {
        return Some(environment_command(
            "linux-change-mirrors",
            "复用笔记换源脚本",
            "mirror",
            &command,
            source_path,
            source_line,
        ));
    }
    if command.contains("LinuxMirrors/raw/main/DockerInstallation.sh") {
        return Some(environment_command(
            "linux-docker-installation",
            "复用笔记 Docker/Compose 安装脚本",
            "docker",
            &command,
            source_path,
            source_line,
        ));
    }
    if command.contains("https://get.docker.com") {
        return Some(environment_command(
            "linux-docker-aliyun-fallback",
            "复用笔记 Docker Aliyun 安装脚本",
            "docker",
            &command,
            source_path,
            source_line,
        ));
    }
    if command == "ssh-keygen -A" {
        return Some(environment_command(
            "ssh-generate-host-keys",
            "复用笔记 SSH host key 初始化",
            "ssh",
            &command,
            source_path,
            source_line,
        ));
    }
    if command.contains("PasswordAuthentication yes") {
        return Some(environment_command(
            "ssh-enable-password-auth",
            "复用笔记 SSH 密码认证配置",
            "ssh",
            &command,
            source_path,
            source_line,
        ));
    }
    if command.contains("PermitRootLogin yes") {
        return Some(environment_command(
            "ssh-enable-root-login",
            "复用笔记 SSH root 登录配置",
            "ssh",
            &command,
            source_path,
            source_line,
        ));
    }
    None
}

fn cleanup_markdown_line(raw_line: &str) -> Option<String> {
    let mut line = raw_line.trim().trim_start_matches('-').trim().to_string();
    if line.is_empty() || line.starts_with('#') || line.starts_with("```") {
        return None;
    }
    if let Some((_, command)) = line.split_once(':')
        && command.contains("curl")
    {
        line = command.trim().to_string();
    }
    if line.contains("[") && line.contains("](") && line.contains(")") {
        line = line.replace('[', "").replace(']', "").replace('(', "").replace(')', "");
    }
    let command = normalize_shell_snippet(&line);
    if command.is_empty() {
        None
    } else {
        Some(command)
    }
}

fn normalize_shell_snippet(raw_command: &str) -> String {
    let mut command = raw_command.trim().trim_end_matches('\\').trim().to_string();
    loop {
        let trimmed = command.trim_end();
        if let Some(prefix) = trimmed.strip_suffix("&&") {
            command = prefix.trim_end().to_string();
            continue;
        }
        if let Some(prefix) = trimmed.strip_suffix(';') {
            command = prefix.trim_end().to_string();
            continue;
        }
        return trimmed.to_string();
    }
}

fn environment_command(
    id: &str,
    label: &str,
    stage: &str,
    command: &str,
    source_path: &str,
    source_line: usize,
) -> EnvironmentSetupCommand {
    EnvironmentSetupCommand {
        id: id.to_string(),
        label: label.to_string(),
        stage: stage.to_string(),
        command: command.to_string(),
        source_path: source_path.to_string(),
        source_line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_reusable_commands_from_environment_notes() {
        let documents = vec![MarkdownDocument {
            source_path: "/tmp/docker环境搭建手册.md".to_string(),
            text: r#"
# 安装compose
bash <(curl -sSL https://gitee.com/SuperManito/LinuxMirrors/raw/main/DockerInstallation.sh)
# centos或ubuntu换源
bash <(curl -sSL https://gitee.com/SuperManito/LinuxMirrors/raw/main/ChangeMirrors.sh)
ssh-keygen -A
sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config && \
"#
            .to_string(),
        }];

        let commands = collect_commands_from_documents(&documents);
        let ids = commands.iter().map(|command| command.id.as_str()).collect::<Vec<_>>();

        // 关键断言：插件应从既有环境搭建笔记提取命令，而不是在 Rust 里复制维护脚本。
        assert!(ids.contains(&"linux-docker-installation"));
        assert!(ids.contains(&"linux-change-mirrors"));
        assert!(ids.contains(&"ssh-generate-host-keys"));
        let password_auth_command = commands
            .iter()
            .find(|command| command.id == "ssh-enable-password-auth")
            .map(|command| command.command.as_str());
        // 关键断言：复用笔记时要去掉 markdown 续行残留，避免远端脚本执行半截命令。
        assert_eq!(
            password_auth_command,
            Some("sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config")
        );
    }
}
