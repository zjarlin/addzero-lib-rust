use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{DotfilesError, Result, io_error};
use crate::settings::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformType {
    Windows,
    Macos,
    Linux,
    Unknown,
}

impl PlatformType {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Macos,
            "linux" => Self::Linux,
            _ => Self::Unknown,
        }
    }

    pub fn is_unix(self) -> bool {
        matches!(self, Self::Macos | Self::Linux)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncType {
    Git,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkLine {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlatformConfig {
    #[serde(
        rename = "packageManager",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub package_manager: Option<String>,
    #[serde(rename = "defaultPackages", default)]
    pub default_packages: BTreeSet<String>,
    #[serde(default)]
    pub links: BTreeSet<LinkLine>,
    #[serde(rename = "nativatePkgRecord", default)]
    pub nativate_pkg_record: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub sync_dir: String,
    pub sync_type: SyncType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_url: Option<String>,
    pub linux_config: PlatformConfig,
    pub mac_config: PlatformConfig,
    pub windows_config: PlatformConfig,
}

impl Config {
    pub fn default_for_settings(settings: &Settings) -> Self {
        Self {
            sync_dir: settings.sync_dir.to_string_lossy().into_owned(),
            sync_type: SyncType::Git,
            cloud_url: None,
            linux_config: PlatformConfig {
                default_packages: default_unix_packages(),
                ..PlatformConfig::default()
            },
            mac_config: PlatformConfig {
                package_manager: Some("brew".to_string()),
                default_packages: default_unix_packages(),
                ..PlatformConfig::default()
            },
            windows_config: PlatformConfig {
                package_manager: Some("winget".to_string()),
                default_packages: default_windows_packages(),
                ..PlatformConfig::default()
            },
        }
    }

    pub fn current_platform_config(&self, platform: PlatformType) -> &PlatformConfig {
        match platform {
            PlatformType::Windows => &self.windows_config,
            PlatformType::Macos => &self.mac_config,
            PlatformType::Linux | PlatformType::Unknown => &self.linux_config,
        }
    }

    pub fn current_platform_config_mut(&mut self, platform: PlatformType) -> &mut PlatformConfig {
        match platform {
            PlatformType::Windows => &mut self.windows_config,
            PlatformType::Macos => &mut self.mac_config,
            PlatformType::Linux | PlatformType::Unknown => &mut self.linux_config,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load_or_init(&self, settings: &Settings) -> Result<Config> {
        if self.path.exists() {
            let content =
                std::fs::read_to_string(&self.path).map_err(|err| io_error(&self.path, err))?;
            let config = serde_json::from_str(&content).map_err(|err| DotfilesError::Json {
                path: self.path.clone(),
                source: err,
            })?;
            return Ok(config);
        }

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
        }
        let config = Config::default_for_settings(settings);
        self.save(&config)?;
        Ok(config)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&self.path, format!("{content}\n"))
            .map_err(|err| io_error(&self.path, err))?;
        Ok(())
    }
}

fn default_unix_packages() -> BTreeSet<String> {
    [
        "git", "curl", "wget", "zsh", "neovim", "node", "npm", "yarn", "qq", "wechat", "utools",
        "bcut",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_windows_packages() -> BTreeSet<String> {
    [
        "pnpm.pnpm",
        "Anysphere.Cursor",
        "Git.Git",
        "Microsoft.WindowsTerminal",
        "Google.Chrome",
        "Microsoft.VisualStudioCode",
        "JetBrains.Toolbox",
        "ClashVergeRev.ClashVergeRev",
        "Tencent.WeChat",
        "Tencent.QQ",
        "OpenJS.NodeJS",
        "JetBrains.IntelliJIDEA.Ultimate",
        "liule.Snipaste",
        "Yuanli.uTools",
        "Ruihu.Apifox",
        "GeekUninstaller.GeekUninstaller",
        "RustDesk.RustDesk",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_dfctx_config_shape() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let settings = Settings::for_home(temp.path());
        let config = Config::default_for_settings(&settings);

        assert_eq!(config.sync_type, SyncType::Git);
        assert!(config.sync_dir.ends_with(".config/df/dfctx"));
        assert!(config.mac_config.default_packages.contains("git"));
        assert!(config.windows_config.default_packages.contains("Git.Git"));
    }

    #[test]
    fn config_store_creates_parent_directory_and_file() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let settings = Settings::for_home(temp.path());
        let store = ConfigStore::new(settings.config_file.clone());

        let config = store.load_or_init(&settings).expect("load config");

        assert_eq!(config.sync_type, SyncType::Git);
        assert!(settings.config_file.exists());
    }
}
