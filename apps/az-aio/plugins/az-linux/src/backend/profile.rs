//! Linux 环境适配器抽象。
//!
//! AIO 客户端先只落 Ubuntu 适配器，但通过 trait 把发行版差异、包管理器和引导
//! 脚本生成隔离出来，后续 Debian、CentOS 或服务器版 CLI 不需要改页面层。

use anyhow::{Result, bail};

use crate::backend::{
    contract::{
        BootstrapPlan, BootstrapPlanRequest, BootstrapStep, CONTRACT_VERSION,
        EnvironmentSetupCatalog, EnvironmentSetupCommand, LinuxDistribution, LinuxProfileSummary,
        SshConfigPreview,
    },
    env_notes::{catalog_command, load_environment_setup_catalog, setup_source_summary},
    script::build_remote_bootstrap_script,
    time::current_timestamp_millis,
};

pub trait LinuxEnvironmentAdapter: Send + Sync {
    fn profile(&self) -> LinuxProfileSummary;

    fn build_plan(&self, request: BootstrapPlanRequest) -> Result<BootstrapPlan>;

    fn build_script(&self, request: BootstrapPlanRequest) -> Result<String>;
}

#[derive(Clone, Debug, Default)]
pub struct UbuntuEnvironmentAdapter;

impl LinuxEnvironmentAdapter for UbuntuEnvironmentAdapter {
    fn profile(&self) -> LinuxProfileSummary {
        LinuxProfileSummary {
            distribution: LinuxDistribution::Ubuntu,
            label: "Ubuntu".to_string(),
            package_manager: "apt".to_string(),
            default_user: "ubuntu".to_string(),
            supported_steps: vec![
                "curl-bootstrap".to_string(),
                "openssh-server".to_string(),
                "authorized-keys".to_string(),
                "client-pairing-file".to_string(),
            ],
        }
    }

    fn build_plan(&self, request: BootstrapPlanRequest) -> Result<BootstrapPlan> {
        validate_request(&request)?;
        let ssh_config = build_ssh_config(&request);
        let catalog = load_environment_setup_catalog();
        let manual_curl_command = build_manual_curl_command(&request);
        let reused_commands = reused_commands_for_ubuntu(&catalog);
        let mut warnings = Vec::new();

        if request.client.public_key.is_none() {
            warnings.push(
                "未提供 publicKey，远端脚本只会准备 SSH 服务，不会自动写入 authorized_keys。"
                    .to_string(),
            );
        }
        if request.client.client_endpoint.contains("<") {
            warnings.push("clientEndpoint 仍是占位符，远端服务器需要能访问真实 AIO 地址。".to_string());
        }
        if reused_commands.is_empty() {
            warnings.push("未读取到环境搭建笔记命令，请检查 /Users/zjarlin/aio/note/环境搭建。".to_string());
        }

        Ok(BootstrapPlan {
            contract_version: CONTRACT_VERSION.to_string(),
            target: request.target.clone(),
            client: request.client.clone(),
            manual_curl_command,
            ssh_config,
            steps: ubuntu_steps(&request, &reused_commands),
            warnings,
            setup_source: setup_source_summary(&catalog),
            reused_commands,
            updated_at_ms: current_timestamp_millis(),
        })
    }

    fn build_script(&self, request: BootstrapPlanRequest) -> Result<String> {
        validate_request(&request)?;
        let catalog = load_environment_setup_catalog();
        let reused_commands = reused_commands_for_ubuntu(&catalog);
        build_remote_bootstrap_script(&request, &reused_commands)
    }
}

pub fn supported_profiles() -> Vec<LinuxProfileSummary> {
    vec![UbuntuEnvironmentAdapter.profile()]
}

pub fn adapter_for(distribution: LinuxDistribution) -> Box<dyn LinuxEnvironmentAdapter> {
    match distribution {
        LinuxDistribution::Ubuntu => Box::new(UbuntuEnvironmentAdapter),
    }
}

fn validate_request(request: &BootstrapPlanRequest) -> Result<()> {
    if request.target.host.trim().is_empty() {
        bail!("target.host must not be blank");
    }
    if request.target.user.trim().is_empty() {
        bail!("target.user must not be blank");
    }
    if request.target.port == 0 {
        bail!("target.port must be greater than 0");
    }
    if request.client.client_name.trim().is_empty() {
        bail!("client.clientName must not be blank");
    }
    if request.client.pair_token.trim().is_empty() {
        bail!("client.pairToken must not be blank");
    }
    if request.install_base_url.trim().is_empty() {
        bail!("installBaseUrl must not be blank");
    }
    Ok(())
}

fn build_ssh_config(request: &BootstrapPlanRequest) -> SshConfigPreview {
    let alias = ssh_alias(&request.target.host);
    let identity_file = "~/.ssh/linux_client_ed25519";
    let config_block = format!(
        "Host {alias}\n  HostName {host}\n  User {user}\n  Port {port}\n  IdentityFile {identity_file}\n  IdentitiesOnly yes\n  ServerAliveInterval 30\n",
        alias = alias,
        host = request.target.host,
        user = request.target.user,
        port = request.target.port,
        identity_file = identity_file,
    );
    let authorized_keys_command = match request.client.public_key.as_deref() {
        Some(public_key) => format!(
            "mkdir -p ~/.ssh && chmod 700 ~/.ssh && grep -qxF {key} ~/.ssh/authorized_keys 2>/dev/null || printf '%s\\n' {key} >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
            key = shell_single_quote(public_key),
        ),
        None => "cat ~/.ssh/linux_client_ed25519.pub | ssh ubuntu@<host> 'mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys'".to_string(),
    };
    let keygen_command = format!(
        "ssh-keygen -t ed25519 -f {identity_file} -C linux-client -N ''",
        identity_file = identity_file,
    );

    SshConfigPreview {
        host_alias: alias,
        config_block,
        authorized_keys_command,
        keygen_command,
    }
}

fn ubuntu_steps(
    request: &BootstrapPlanRequest,
    reused_commands: &[EnvironmentSetupCommand],
) -> Vec<BootstrapStep> {
    let mut steps = vec![
        BootstrapStep {
            id: "manual-curl".to_string(),
            label: "远端执行 curl".to_string(),
            description: "初始不可配对时，先在 Ubuntu 服务器终端手动执行客户端生成的 curl 脚本。".to_string(),
            command: Some(build_manual_curl_command(request)),
            manual: true,
        },
        BootstrapStep {
            id: "ssh-service".to_string(),
            label: "开启 SSH 服务".to_string(),
            description: "脚本会尝试 systemctl enable --now ssh，失败时降级 service ssh start。".to_string(),
            command: Some("sudo systemctl enable --now ssh || sudo service ssh start".to_string()),
            manual: false,
        },
        BootstrapStep {
            id: "pairing-file".to_string(),
            label: "写入配对种子".to_string(),
            description: "脚本把客户端名称、访问端点和 pairToken 写入 ~/.config/linux/client-pairing.json。".to_string(),
            command: Some(format!(
                "cat ~/.config/linux/client-pairing.json # {}",
                request.client.client_name,
            )),
            manual: false,
        },
    ];

    for command in reused_commands {
        steps.push(BootstrapStep {
            id: format!("reuse-{}", command.id),
            label: command.label.clone(),
            description: format!(
                "来自 {}:{}，客户端只编排复用，不复制维护。",
                command.source_path, command.source_line
            ),
            command: Some(command.command.clone()),
            manual: false,
        });
    }

    steps
}

fn reused_commands_for_ubuntu(catalog: &EnvironmentSetupCatalog) -> Vec<EnvironmentSetupCommand> {
    [
        "linux-change-mirrors",
        "linux-docker-installation",
        "ssh-generate-host-keys",
        "ssh-enable-password-auth",
    ]
    .into_iter()
    .filter_map(|id| catalog_command(catalog, id).cloned())
    .collect()
}

pub fn build_manual_curl_command(request: &BootstrapPlanRequest) -> String {
    let mut url = format!(
        "{}/api/linux/bootstrap-script?distribution={}&targetHost={}&targetUser={}&port={}&clientName={}&clientEndpoint={}&pairToken={}",
        request.install_base_url.trim_end_matches('/'),
        request.target.distribution.id(),
        query_escape(&request.target.host),
        query_escape(&request.target.user),
        request.target.port,
        query_escape(&request.client.client_name),
        query_escape(&request.client.client_endpoint),
        query_escape(&request.client.pair_token),
    );
    if let Some(public_key) = request.client.public_key.as_deref() {
        url.push_str("&publicKey=");
        url.push_str(&query_escape(public_key));
    }
    format!("curl -fsSL {} | bash", shell_single_quote(&url))
}

fn ssh_alias(host: &str) -> String {
    let normalized = host
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if normalized.is_empty() {
        "linux-host".to_string()
    } else {
        format!("linux-{normalized}")
    }
}

fn query_escape(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::contract::{ClientPairingSeed, LinuxTarget};

    #[test]
    fn ubuntu_plan_contains_manual_curl_and_ssh_config() -> anyhow::Result<()> {
        let adapter = UbuntuEnvironmentAdapter;
        let request = BootstrapPlanRequest {
            target: LinuxTarget {
                host: "192.168.31.100".to_string(),
                port: 22,
                user: "ubuntu".to_string(),
                distribution: LinuxDistribution::Ubuntu,
            },
            client: ClientPairingSeed {
                client_name: "aio".to_string(),
                client_endpoint: "http://192.168.31.10:18080".to_string(),
                pair_token: "demo-token".to_string(),
                public_key: Some("ssh-ed25519 AAAA demo".to_string()),
            },
            install_base_url: "http://192.168.31.10:18080".to_string(),
        };

        let plan = adapter.build_plan(request)?;

        // 关键断言：初始不可配对时，计划必须给出可手动复制到远端执行的 curl 命令。
        assert!(plan.manual_curl_command.contains("curl -fsSL"));
        assert!(plan.manual_curl_command.contains("/api/linux/bootstrap-script"));
        assert!(plan.ssh_config.config_block.contains("Host linux-192-168-31-100"));
        assert_eq!(plan.setup_source.source_root, "/Users/zjarlin/aio/note/环境搭建");
        Ok(())
    }

    #[test]
    fn shell_quote_keeps_single_quote_safe() {
        assert_eq!(shell_single_quote("a'b"), "'a'\\''b'");
    }
}
