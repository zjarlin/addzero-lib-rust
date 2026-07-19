use anyhow::Result;

use crate::{
    command::{CommandRequest, CommandRunner},
    config::RelayConfig,
};

const ADMIN_SHELL: &str = "if [ \"$(id -u)\" -eq 0 ]; then exec sh -s; else exec sudo -n sh -s; fi";
const NPM_PACKAGE: &str = "addhost-cli";

pub(crate) fn prepare(runner: &dyn CommandRunner, config: &RelayConfig) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let script = format!(
        r#"set -eu
command -v npm >/dev/null 2>&1 || {{
    echo '公网机需要先安装 Node.js 和 npm' >&2
    exit 1
}}
npm install --global --no-audit --no-fund '{NPM_PACKAGE}@{version}'
binary=$(command -v addhost)
install -d -m 0755 /etc/addhost
if [ ! -f /etc/addhost/routes.toml ]; then
    printf 'routes = {{}}\n' > /etc/addhost/routes.toml
fi
cat > /etc/systemd/system/addhost-relay.service <<UNIT
[Unit]
Description=addhost HTTP relay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=$binary relay serve --listen 0.0.0.0:80 --routes /etc/addhost/routes.toml
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable --now addhost-relay
systemctl restart addhost-relay
if ! systemctl is-active --quiet addhost-relay; then
    journalctl -u addhost-relay --no-pager -n 50 >&2
    exit 1
fi
"#
    );

    run_admin_script(runner, config, &script)?.ensure_success("初始化公网 addhost relay")
}

pub(crate) fn install_route(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    host: &str,
    remote_port: u16,
) -> Result<()> {
    let script = format!(
        "set -eu\nbinary=$(command -v addhost)\n\"$binary\" relay route set '{host}' {remote_port}\n"
    );
    run_admin_script(runner, config, &script)?.ensure_success("发布公网域名路由")
}

pub(crate) fn remove_route(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    host: &str,
) -> Result<()> {
    let script =
        format!("set -eu\nbinary=$(command -v addhost)\n\"$binary\" relay route remove '{host}'\n");
    run_admin_script(runner, config, &script)?.ensure_success("删除公网域名路由")
}

fn run_admin_script(
    runner: &dyn CommandRunner,
    config: &RelayConfig,
    script: &str,
) -> Result<crate::command::CommandOutput> {
    let args = vec![config.server.clone(), ADMIN_SHELL.to_owned()];
    runner.run(CommandRequest {
        program: "ssh",
        args: &args,
        stdin: Some(script),
    })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use anyhow::Result;

    use crate::command::{CommandOutput, CommandRequest};

    use super::*;

    struct RecordingRunner {
        script: RefCell<String>,
    }

    impl CommandRunner for RecordingRunner {
        fn run(&self, request: CommandRequest<'_>) -> Result<CommandOutput> {
            self.script
                .replace(request.stdin.map_or_else(String::new, str::to_owned));
            Ok(CommandOutput {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    #[test]
    fn route_is_registered_by_remote_binary() -> Result<()> {
        let runner = RecordingRunner {
            script: RefCell::new(String::new()),
        };
        let config = RelayConfig::create("root@example.com".to_owned(), "example.com".to_owned())?;
        install_route(&runner, &config, "demo.example.com", 23_456)?;

        let script = runner.script.borrow();
        assert!(script.contains("relay route set 'demo.example.com' 23456"));
        Ok(())
    }
}
