use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{AzGitError, GitHostingProvider, Result};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitAccountConfig {
    #[serde(default)]
    pub github_username: String,
    #[serde(default)]
    pub gitee_username: String,
    #[serde(default)]
    pub gitlab_username: String,
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
    fn config_store_round_trips_usernames() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let store = GitAccountConfigStore::new(temp.path().join("accounts.json"));
        let mut config = GitAccountConfig::default();
        config.set_username(GitHostingProvider::GitHub, " zjarlin ");
        config.set_username(GitHostingProvider::Gitee, "gitee-user");

        store.save(&config).expect("save config");
        let loaded = store.load().expect("load config");

        assert_eq!(
            loaded.configured_username(GitHostingProvider::GitHub),
            Some("zjarlin".to_string())
        );
        assert_eq!(loaded.username(GitHostingProvider::GitLab), "");
    }
}
