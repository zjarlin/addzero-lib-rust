use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{AzGitError, GitHostingProvider, GitRemoteRepository, Result};

/// Default root used when binding remote repositories to local project paths.
pub const DEFAULT_SYNC_WORKSPACE: &str = "~/az-sync/workspace";

/// Local account and project binding configuration for Git hosting providers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitAccountConfig {
    /// Preferred GitHub username when command-line login discovery is unavailable.
    #[serde(default)]
    pub github_username: String,
    /// Preferred Gitee username when provider-specific discovery is unavailable.
    #[serde(default)]
    pub gitee_username: String,
    /// Preferred GitLab username when provider-specific discovery is unavailable.
    #[serde(default)]
    pub gitlab_username: String,
    /// Root directory where discovered repositories are mapped.
    #[serde(default = "default_sync_workspace")]
    pub sync_workspace: String,
    /// Repositories discovered from a logged-in account and bound to local paths.
    #[serde(default)]
    pub project_bindings: Vec<GitProjectBinding>,
}

impl Default for GitAccountConfig {
    fn default() -> Self {
        Self {
            github_username: String::new(),
            gitee_username: String::new(),
            gitlab_username: String::new(),
            sync_workspace: default_sync_workspace(),
            project_bindings: Vec::new(),
        }
    }
}

/// Local binding for one remote Git repository.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitProjectBinding {
    /// Hosting provider that owns the repository.
    pub provider: GitHostingProvider,
    /// Repository owner or organization login.
    pub owner: String,
    /// Repository name without the owner prefix.
    pub name: String,
    /// Clone-capable remote URL, preferring SSH when the provider exposes it.
    pub remote_url: String,
    /// Local path generated from the configured sync workspace and repository identity.
    pub local_path: String,
}

impl GitProjectBinding {
    /// Creates a local project binding from a discovered remote repository.
    pub fn from_remote(repository: GitRemoteRepository, sync_workspace: &str) -> Self {
        Self {
            provider: repository.provider,
            owner: repository.owner.clone(),
            name: repository.name.clone(),
            remote_url: repository.remote_url,
            local_path: local_project_path(sync_workspace, &repository.owner, &repository.name),
        }
    }

    /// Returns the provider-native `owner/name` display identity.
    pub fn name_with_owner(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

impl GitAccountConfig {
    pub fn username(&self, provider: GitHostingProvider) -> &str {
        match provider {
            GitHostingProvider::GitHub => &self.github_username,
            GitHostingProvider::Gitee => &self.gitee_username,
            GitHostingProvider::GitLab => &self.gitlab_username,
        }
    }

    pub fn set_username(&mut self, provider: GitHostingProvider, username: impl Into<String>) {
        let username = username.into().trim().to_string();
        match provider {
            GitHostingProvider::GitHub => self.github_username = username,
            GitHostingProvider::Gitee => self.gitee_username = username,
            GitHostingProvider::GitLab => self.gitlab_username = username,
        }
    }

    pub fn configured_username(&self, provider: GitHostingProvider) -> Option<String> {
        let username = self.username(provider).trim();
        (!username.is_empty()).then(|| username.to_string())
    }

    /// Returns the configured sync workspace or the built-in default when blank.
    pub fn sync_workspace(&self) -> &str {
        let workspace = self.sync_workspace.trim();
        if workspace.is_empty() {
            DEFAULT_SYNC_WORKSPACE
        } else {
            workspace
        }
    }

    /// Updates the sync workspace and rebinds existing project paths.
    pub fn set_sync_workspace(&mut self, sync_workspace: impl Into<String>) {
        let sync_workspace = sync_workspace.into().trim().to_string();
        self.sync_workspace = if sync_workspace.is_empty() {
            default_sync_workspace()
        } else {
            sync_workspace
        };
        self.rebind_project_paths();
    }

    /// Replaces all stored project bindings.
    pub fn set_project_bindings(&mut self, project_bindings: Vec<GitProjectBinding>) {
        self.project_bindings = project_bindings;
    }

    /// Binds discovered repositories under the current sync workspace.
    pub fn bind_remote_repositories(&mut self, repositories: Vec<GitRemoteRepository>) {
        let sync_workspace = self.sync_workspace().to_string();
        self.project_bindings = repositories
            .into_iter()
            .map(|repository| GitProjectBinding::from_remote(repository, &sync_workspace))
            .collect();
    }

    fn rebind_project_paths(&mut self) {
        let sync_workspace = self.sync_workspace().to_string();
        for project in &mut self.project_bindings {
            project.local_path = local_project_path(&sync_workspace, &project.owner, &project.name);
        }
    }
}

/// Returns the default sync workspace as an owned string for serde defaults.
pub fn default_sync_workspace() -> String {
    DEFAULT_SYNC_WORKSPACE.to_string()
}

fn local_project_path(sync_workspace: &str, owner: &str, name: &str) -> String {
    let workspace = sync_workspace.trim().trim_end_matches('/');
    format!("{workspace}/{owner}/{name}")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitAccountConfigStore {
    path: PathBuf,
}

impl GitAccountConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or(AzGitError::ConfigDirUnavailable)?;
        Ok(config_dir
            .join("addzero")
            .join("az-git")
            .join("accounts.json"))
    }

    pub fn default_store() -> Result<Self> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<GitAccountConfig> {
        if !self.path.exists() {
            return Ok(GitAccountConfig::default());
        }

        let content = fs::read_to_string(&self.path).map_err(|source| AzGitError::ReadConfig {
            path: self.path.clone(),
            source,
        })?;
        serde_json::from_str(&content).map_err(|source| AzGitError::ParseConfig {
            path: self.path.clone(),
            source,
        })
    }

    pub fn save(&self, config: &GitAccountConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| AzGitError::WriteConfig {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.path, content).map_err(|source| AzGitError::WriteConfig {
            path: self.path.clone(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_store_round_trips_project_defaults() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let store = GitAccountConfigStore::new(temp.path().join("accounts.json"));
        let mut config = GitAccountConfig::default();
        config.set_username(GitHostingProvider::GitHub, " zjarlin ");
        config.set_username(GitHostingProvider::Gitee, "gitee-user");
        config.set_sync_workspace("~/az-sync/workspace");
        config.bind_remote_repositories(vec![GitRemoteRepository {
            provider: GitHostingProvider::GitHub,
            owner: "zjarlin".to_string(),
            name: "addzero-lib-rust".to_string(),
            remote_url: "git@github.com:zjarlin/addzero-lib-rust.git".to_string(),
        }]);

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(
            loaded.configured_username(GitHostingProvider::GitHub),
            Some("zjarlin".to_string())
        );
        assert_eq!(loaded.username(GitHostingProvider::GitLab), "");
        assert_eq!(loaded.sync_workspace(), "~/az-sync/workspace");
        assert_eq!(
            loaded.project_bindings[0].local_path,
            "~/az-sync/workspace/zjarlin/addzero-lib-rust"
        );
    }

    #[test]
    fn sync_workspace_rebinds_existing_project_paths() {
        let mut config = GitAccountConfig::default();
        config.bind_remote_repositories(vec![GitRemoteRepository {
            provider: GitHostingProvider::GitHub,
            owner: "zjarlin".to_string(),
            name: "sub2api".to_string(),
            remote_url: "git@github.com:zjarlin/sub2api.git".to_string(),
        }]);

        config.set_sync_workspace("~/work");

        assert_eq!(
            config.project_bindings[0].local_path,
            "~/work/zjarlin/sub2api"
        );
    }
}
