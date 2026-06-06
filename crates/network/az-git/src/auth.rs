use std::{
    collections::HashMap,
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{AzGitError, GitAccountConfig, GitHostingProvider, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMethod {
    GhCli,
    Web,
    Token,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthState {
    Connected,
    Available,
    NotDetected,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    pub method: AuthMethod,
    pub state: AuthState,
    pub username: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthLoginFlow {
    pub method: AuthMethod,
    pub label: String,
    pub url: Option<String>,
    pub command: Option<Vec<String>>,
    pub stores_secret: bool,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitHostingAccountStatus {
    pub provider: GitHostingProvider,
    pub configured_username: Option<String>,
    pub sessions: Vec<AuthSession>,
    pub login_flows: Vec<AuthLoginFlow>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthDiscoveryOptions {
    pub config: GitAccountConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    pub status_success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemCommandRunner;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(25);

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let mut child =
            Command::new(program)
                .args(args)
                .spawn()
                .map_err(|source| AzGitError::Command {
                    program: program.to_string(),
                    source,
                })?;
        let started_at = Instant::now();

        loop {
            match child.try_wait().map_err(|source| AzGitError::Command {
                program: program.to_string(),
                source,
            })? {
                Some(_) => {
                    let output =
                        child
                            .wait_with_output()
                            .map_err(|source| AzGitError::Command {
                                program: program.to_string(),
                                source,
                            })?;

                    return Ok(CommandOutput {
                        status_success: output.status.success(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    });
                }
                None if started_at.elapsed() >= COMMAND_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AzGitError::CommandTimeout {
                        program: program.to_string(),
                        timeout_ms: COMMAND_TIMEOUT.as_millis() as u64,
                    });
                }
                None => thread::sleep(COMMAND_POLL_INTERVAL),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuthDiscovery<R = SystemCommandRunner> {
    runner: R,
}

impl AuthDiscovery<SystemCommandRunner> {
    pub fn system() -> Self {
        Self {
            runner: SystemCommandRunner,
        }
    }
}

impl<R> AuthDiscovery<R>
where
    R: CommandRunner,
{
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    pub fn discover_all(&self, options: &AuthDiscoveryOptions) -> Vec<GitHostingAccountStatus> {
        GitHostingProvider::ALL
            .iter()
            .map(|provider| self.discover_provider(*provider, options))
            .collect()
    }

    pub fn discover_provider(
        &self,
        provider: GitHostingProvider,
        options: &AuthDiscoveryOptions,
    ) -> GitHostingAccountStatus {
        GitHostingAccountStatus {
            provider,
            configured_username: options.config.configured_username(provider),
            sessions: self.discover_sessions(provider),
            login_flows: login_flows(provider),
        }
    }

    fn discover_sessions(&self, provider: GitHostingProvider) -> Vec<AuthSession> {
        match provider {
            GitHostingProvider::GitHub => vec![self.discover_github_gh_session()],
            GitHostingProvider::Gitee | GitHostingProvider::GitLab => Vec::new(),
        }
    }

    fn discover_github_gh_session(&self) -> AuthSession {
        let output = match self
            .runner
            .run("gh", &["auth", "status", "--json", "hosts"])
        {
            Ok(output) => output,
            Err(error) => {
                return AuthSession {
                    method: AuthMethod::GhCli,
                    state: AuthState::NotDetected,
                    username: None,
                    source: None,
                    message: format!("未检测到 gh 命令行或无法执行：{error}"),
                };
            }
        };

        match parse_gh_auth_status(&output.stdout) {
            Some(account) => AuthSession {
                method: AuthMethod::GhCli,
                state: account.auth_state(),
                username: Some(account.login.clone()),
                source: account
                    .token_source
                    .as_deref()
                    .map(|source| format!("gh:{source}")),
                message: account.message(),
            },
            None if output.status_success => AuthSession {
                method: AuthMethod::GhCli,
                state: AuthState::NotDetected,
                username: None,
                source: Some("gh".to_string()),
                message: "已检测到 gh，但没有可用的 github.com 登录态。".to_string(),
            },
            None => AuthSession {
                method: AuthMethod::GhCli,
                state: AuthState::Error,
                username: None,
                source: Some("gh".to_string()),
                message: output.stderr.trim().to_string(),
            },
        }
    }
}

fn login_flows(provider: GitHostingProvider) -> Vec<AuthLoginFlow> {
    let info = provider.info();
    let mut flows = Vec::new();

    if provider == GitHostingProvider::GitHub {
        flows.push(AuthLoginFlow {
            method: AuthMethod::GhCli,
            label: "复用 gh 登录态".to_string(),
            url: None,
            command: Some(vec![
                "gh".to_string(),
                "auth".to_string(),
                "login".to_string(),
                "--hostname".to_string(),
                info.host.to_string(),
            ]),
            stores_secret: false,
            description: "检测到 gh 后优先复用本机系统凭据中的 GitHub 登录态。".to_string(),
        });
    }

    flows.push(AuthLoginFlow {
        method: AuthMethod::Web,
        label: "网页登录".to_string(),
        url: Some(info.web_login_url.to_string()),
        command: None,
        stores_secret: false,
        description: format!("打开 {} 的网页登录入口。", info.label),
    });
    flows.push(AuthLoginFlow {
        method: AuthMethod::Token,
        label: "令牌登录".to_string(),
        url: Some(info.token_url.to_string()),
        command: None,
        stores_secret: true,
        // 令牌必须先接入系统凭据存储，才允许进入持久化路径。
        description: "本版只提供令牌入口，不把令牌明文写入配置文件。".to_string(),
    });

    flows
}

#[derive(Debug, Deserialize)]
struct GhAuthStatus {
    hosts: HashMap<String, Vec<GhAccount>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhAccount {
    state: String,
    active: bool,
    login: String,
    error: Option<String>,
    token_source: Option<String>,
    scopes: Option<String>,
    git_protocol: Option<String>,
}

impl GhAccount {
    fn auth_state(&self) -> AuthState {
        if self.state == "success" {
            AuthState::Connected
        } else {
            AuthState::Available
        }
    }

    fn message(&self) -> String {
        let protocol = self.git_protocol.as_deref().unwrap_or("未知");
        if self.state == "success" {
            let scopes = self.scopes.as_deref().unwrap_or("未知");
            format!("已复用 gh 登录态，协议：{protocol}；权限范围：{scopes}")
        } else {
            let error = self.error.as_deref().unwrap_or("未知状态错误");
            format!(
                "检测到 gh 登录态，协议：{protocol}；状态 {}：{error}",
                self.state
            )
        }
    }
}

fn parse_gh_auth_status(stdout: &str) -> Option<GhAccount> {
    let status = serde_json::from_str::<GhAuthStatus>(stdout).ok()?;
    status
        .hosts
        .get("github.com")?
        .iter()
        .find(|account| account.active && !account.login.trim().is_empty())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct FakeRunner {
        output: CommandOutput,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> Result<CommandOutput> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn github_uses_active_gh_session_from_json_status() {
        let runner = FakeRunner {
            output: CommandOutput {
                status_success: true,
                stdout: r#"{"hosts":{"github.com":[{"state":"success","active":true,"host":"github.com","login":"zjarlin","error":null,"tokenSource":"keyring","scopes":"repo, workflow","gitProtocol":"https"}]}}"#.to_string(),
                stderr: String::new(),
            },
        };

        let status = AuthDiscovery::new(runner)
            .discover_provider(GitHostingProvider::GitHub, &AuthDiscoveryOptions::default());

        assert_eq!(status.sessions[0].state, AuthState::Connected);
        assert_eq!(status.sessions[0].username.as_deref(), Some("zjarlin"));
        assert!(
            status
                .login_flows
                .iter()
                .any(|flow| flow.method == AuthMethod::GhCli)
        );
    }

    #[test]
    fn github_keeps_active_gh_username_when_status_check_errors() {
        let runner = FakeRunner {
            output: CommandOutput {
                status_success: true,
                stdout: r#"{"hosts":{"github.com":[{"state":"error","error":"Get \"https://api.github.com/\": EOF","active":true,"host":"github.com","login":"zjarlin","tokenSource":"keyring","gitProtocol":"https"}]}}"#.to_string(),
                stderr: String::new(),
            },
        };

        let status = AuthDiscovery::new(runner)
            .discover_provider(GitHostingProvider::GitHub, &AuthDiscoveryOptions::default());

        assert_eq!(status.sessions[0].state, AuthState::Available);
        assert_eq!(status.sessions[0].username.as_deref(), Some("zjarlin"));
    }

    #[test]
    fn non_github_providers_expose_web_and_token_flows() {
        let runner = FakeRunner {
            output: CommandOutput {
                status_success: true,
                stdout: "{}".to_string(),
                stderr: String::new(),
            },
        };

        let status = AuthDiscovery::new(runner)
            .discover_provider(GitHostingProvider::GitLab, &AuthDiscoveryOptions::default());

        assert!(status.sessions.is_empty());
        assert_eq!(
            status
                .login_flows
                .iter()
                .map(|flow| flow.method)
                .collect::<Vec<_>>(),
            vec![AuthMethod::Web, AuthMethod::Token]
        );
    }
}
