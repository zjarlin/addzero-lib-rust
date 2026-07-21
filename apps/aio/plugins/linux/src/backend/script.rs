//! 远端 Ubuntu 引导脚本渲染。
//!
//! 这个脚本是客户端插件先行阶段的临时远端入口：它不假设服务器 CLI 已存在，只负责
//! 准备基础联网、SSH 服务和配对种子文件。服务器版 CLI 完成后可以替换脚本主体。

use anyhow::Result;

use crate::backend::{
    contract::{BootstrapPlanRequest, CONTRACT_VERSION, EnvironmentSetupCommand, RemotePairingFile},
    profile::shell_single_quote,
    time::current_timestamp_millis,
};

pub fn build_remote_bootstrap_script(
    request: &BootstrapPlanRequest,
    reused_commands: &[EnvironmentSetupCommand],
) -> Result<String> {
    let generated_at_ms = current_timestamp_millis();
    let pairing_file = RemotePairingFile {
        contract_version: CONTRACT_VERSION.to_string(),
        distribution: request.target.distribution,
        client_name: request.client.client_name.clone(),
        client_endpoint: request.client.client_endpoint.clone(),
        pair_token: request.client.pair_token.clone(),
        generated_at_ms,
    };
    let pairing_json = serde_json::to_string_pretty(&pairing_file)?;
    let authorized_keys_block = authorized_keys_block(request.client.public_key.as_deref());
    let reused_setup_block = reused_setup_block(reused_commands);
    let script = REMOTE_BOOTSTRAP_TEMPLATE
        .replace("__PAIRING_JSON__", &pairing_json)
        .replace("__REUSED_SETUP_BLOCK__", &reused_setup_block)
        .replace("__AUTHORIZED_KEYS_BLOCK__", &authorized_keys_block);

    Ok(script)
}

fn reused_setup_block(reused_commands: &[EnvironmentSetupCommand]) -> String {
    if reused_commands.is_empty() {
        return r#"echo "[linux] no reusable environment setup command found; run ssh baseline only"
$SUDO apt-get update
$SUDO apt-get install -y ca-certificates curl openssh-server"#
            .to_string();
    }

    let mut body = String::new();
    body.push_str("echo \"[linux] run reusable environment setup commands from notes\"\n");
    body.push_str("$SUDO apt-get update\n");
    body.push_str("$SUDO apt-get install -y ca-certificates curl openssh-server\n");
    for command in reused_commands {
        body.push_str(&format!(
            "echo {}\n",
            shell_single_quote(&format!(
                "[linux] reuse {} from {}:{}",
                command.id, command.source_path, command.source_line
            ))
        ));
        body.push_str(&format!(
            "$SUDO bash -lc {}\n",
            shell_single_quote(&command.command)
        ));
    }
    body
}

fn authorized_keys_block(public_key: Option<&str>) -> String {
    match public_key.filter(|value| !value.trim().is_empty()) {
        Some(public_key) => format!(
            r#"mkdir -p "$HOME/.ssh"
chmod 700 "$HOME/.ssh"
touch "$HOME/.ssh/authorized_keys"
if ! grep -qxF {public_key} "$HOME/.ssh/authorized_keys"; then
  printf '%s\n' {public_key} >> "$HOME/.ssh/authorized_keys"
fi
chmod 600 "$HOME/.ssh/authorized_keys""#,
            public_key = shell_single_quote(public_key),
        ),
        None => "echo '[linux] public key not provided; skip authorized_keys update'".to_string(),
    }
}

const REMOTE_BOOTSTRAP_TEMPLATE: &str = r#"#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -eq 0 ]; then
  SUDO=""
else
  SUDO="sudo"
fi

export DEBIAN_FRONTEND=noninteractive
__REUSED_SETUP_BLOCK__

echo "[linux] enable ssh service"
if command -v systemctl >/dev/null 2>&1; then
  $SUDO systemctl enable --now ssh || $SUDO systemctl enable --now sshd || true
fi
if ! pgrep -x sshd >/dev/null 2>&1; then
  $SUDO service ssh start || $SUDO service sshd start || true
fi

echo "[linux] write client pairing seed"
PAIRING_DIR="$HOME/.config/linux"
PAIRING_FILE="$PAIRING_DIR/client-pairing.json"
mkdir -p "$PAIRING_DIR"
cat > "$PAIRING_FILE" <<'AZ_LINUX_PAIRING'
__PAIRING_JSON__
AZ_LINUX_PAIRING
chmod 600 "$PAIRING_FILE"

__AUTHORIZED_KEYS_BLOCK__

echo "[linux] done"
echo "[linux] pairing file: $PAIRING_FILE"
"#;

#[cfg(test)]
mod tests {
    use crate::backend::contract::{
        BootstrapPlanRequest, ClientPairingSeed, EnvironmentSetupCommand, LinuxDistribution, LinuxTarget,
    };

    use super::build_remote_bootstrap_script;

    #[test]
    fn bootstrap_script_contains_pairing_file_and_ubuntu_packages() -> anyhow::Result<()> {
        let reused_commands = vec![EnvironmentSetupCommand {
            id: "linux-docker-installation".to_string(),
            label: "复用笔记 Docker/Compose 安装脚本".to_string(),
            stage: "docker".to_string(),
            command: "bash <(curl -sSL https://gitee.com/SuperManito/LinuxMirrors/raw/main/DockerInstallation.sh)".to_string(),
            source_path: "/tmp/docker环境搭建手册.md".to_string(),
            source_line: 1,
        }];
        let script = build_remote_bootstrap_script(&BootstrapPlanRequest {
            target: LinuxTarget {
                host: "192.168.31.100".to_string(),
                port: 22,
                user: "ubuntu".to_string(),
                distribution: LinuxDistribution::Ubuntu,
            },
            client: ClientPairingSeed {
                client_name: "aio".to_string(),
                client_endpoint: "http://127.0.0.1:18080".to_string(),
                pair_token: "demo".to_string(),
                public_key: Some("ssh-ed25519 AAAA demo".to_string()),
            },
            install_base_url: "http://127.0.0.1:18080".to_string(),
        }, &reused_commands)?;

        // 关键断言：服务器版 CLI 尚未存在时，curl 脚本必须能独立准备 SSH 与配对种子。
        assert!(script.contains("DockerInstallation.sh"));
        assert!(script.contains("client-pairing.json"));
        assert!(script.contains("ssh-ed25519 AAAA demo"));
        Ok(())
    }
}
