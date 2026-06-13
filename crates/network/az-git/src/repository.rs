use serde::{Deserialize, Serialize};

use crate::{
    auth::{CommandRunner, SystemCommandRunner},
    provider::GitHostingProvider,
};

/// Remote repository metadata discovered from a logged-in Git hosting account.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitRemoteRepository {
    /// Hosting provider that returned this repository.
    pub provider: GitHostingProvider,
    /// Repository owner or organization login.
    pub owner: String,
    /// Repository name without the owner prefix.
    pub name: String,
    /// Clone-capable remote URL, preferring SSH when available.
    pub remote_url: String,
}

/// Discovers repositories by reusing provider-specific local login tools.
#[derive(Clone, Debug)]
pub struct GitRepositoryDiscovery<R = SystemCommandRunner> {
    runner: R,
}

impl GitRepositoryDiscovery<SystemCommandRunner> {
    /// Creates a discovery service backed by local system commands.
    pub fn system() -> Self {
        Self {
            runner: SystemCommandRunner,
        }
    }
}

impl<R> GitRepositoryDiscovery<R>
where
    R: CommandRunner,
{
    /// Creates a discovery service backed by a custom command runner.
    pub fn new(runner: R) -> Self {
        Self { runner }
    }

    /// Lists repositories visible to `owner` for the selected provider.
    pub fn discover_provider_repositories(
        &self,
        provider: GitHostingProvider,
        owner: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<GitRemoteRepository>> {
        match provider {
            GitHostingProvider::GitHub => self.discover_github_repositories(owner, limit),
            GitHostingProvider::Gitee | GitHostingProvider::GitLab => Ok(Vec::new()),
        }
    }

    fn discover_github_repositories(
        &self,
        owner: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<GitRemoteRepository>> {
        let owner = owner.trim();
        if owner.is_empty() {
            return Ok(Vec::new());
        }

        let limit = limit.max(1).to_string();
        let output = self.runner.run(
            "gh",
            &[
                "repo",
                "list",
                owner,
                "--limit",
                &limit,
                "--json",
                "name,nameWithOwner,url,sshUrl,owner",
            ],
        )?;

        if !output.status_success {
            anyhow::bail!("命令 gh 执行失败：{}", output.stderr.trim());
        }

        parse_gh_repository_list(&output.stdout)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhRepository {
    name: String,
    name_with_owner: String,
    url: String,
    ssh_url: Option<String>,
    owner: Option<GhRepositoryOwner>,
}

#[derive(Clone, Debug, Deserialize)]
struct GhRepositoryOwner {
    login: String,
}

fn parse_gh_repository_list(stdout: &str) -> anyhow::Result<Vec<GitRemoteRepository>> {
    let repositories = serde_json::from_str::<Vec<GhRepository>>(stdout)
        .map_err(|source| anyhow::anyhow!("解析 gh 输出失败：{source}"))?;

    Ok(repositories
        .into_iter()
        .filter_map(GhRepository::into_remote_repository)
        .collect())
}

impl GhRepository {
    fn into_remote_repository(self) -> Option<GitRemoteRepository> {
        let owner = self
            .owner
            .map(|owner| owner.login)
            .or_else(|| owner_from_name_with_owner(&self.name_with_owner))?;
        let name = if self.name.trim().is_empty() {
            name_from_name_with_owner(&self.name_with_owner)?
        } else {
            self.name
        };
        let remote_url = self
            .ssh_url
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| git_url_from_web_url(&self.url));

        Some(GitRemoteRepository {
            provider: GitHostingProvider::GitHub,
            owner,
            name,
            remote_url,
        })
    }
}

fn owner_from_name_with_owner(name_with_owner: &str) -> Option<String> {
    name_with_owner
        .split_once('/')
        .map(|(owner, _)| owner.trim().to_string())
        .filter(|owner| !owner.is_empty())
}

fn name_from_name_with_owner(name_with_owner: &str) -> Option<String> {
    name_with_owner
        .split_once('/')
        .map(|(_, name)| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn git_url_from_web_url(web_url: &str) -> String {
    let web_url = web_url.trim();
    if web_url.ends_with(".git") {
        web_url.to_string()
    } else {
        format!("{web_url}.git")
    }
}

#[cfg(test)]
mod tests {
    use super::{GitRemoteRepository, GitRepositoryDiscovery, parse_gh_repository_list};
    use crate::{
        auth::{CommandOutput, CommandRunner},
        provider::GitHostingProvider,
    };

    #[derive(Clone, Debug)]
    struct FakeRunner {
        output: CommandOutput,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, _program: &str, _args: &[&str]) -> anyhow::Result<CommandOutput> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn github_repository_discovery_maps_gh_repo_list_output() {
        let runner = FakeRunner {
            output: CommandOutput {
                status_success: true,
                stdout: r#"[{"name":"addzero-lib-rust","nameWithOwner":"zjarlin/addzero-lib-rust","owner":{"login":"zjarlin"},"sshUrl":"git@github.com:zjarlin/addzero-lib-rust.git","url":"https://github.com/zjarlin/addzero-lib-rust"}]"#.to_string(),
                stderr: String::new(),
            },
        };

        let repositories = GitRepositoryDiscovery::new(runner)
            .discover_provider_repositories(GitHostingProvider::GitHub, "zjarlin", 100)
            .expect("discover repositories");

        assert_eq!(
            repositories,
            vec![GitRemoteRepository {
                provider: GitHostingProvider::GitHub,
                owner: "zjarlin".to_string(),
                name: "addzero-lib-rust".to_string(),
                remote_url: "git@github.com:zjarlin/addzero-lib-rust.git".to_string(),
            }]
        );
    }

    #[test]
    fn github_repository_discovery_falls_back_to_https_git_url() {
        let repositories = parse_gh_repository_list(
            r#"[{"name":"sub2api","nameWithOwner":"zjarlin/sub2api","owner":null,"sshUrl":null,"url":"https://github.com/zjarlin/sub2api"}]"#,
        )
        .expect("parse repositories");

        assert_eq!(
            repositories[0].remote_url,
            "https://github.com/zjarlin/sub2api.git"
        );
    }
}
