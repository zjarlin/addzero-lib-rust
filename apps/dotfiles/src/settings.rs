use std::path::{Path, PathBuf};

use crate::config::PlatformType;
use crate::error::{Result, io_error, message};

#[derive(Clone, Debug)]
pub struct Settings {
    pub home_dir: PathBuf,
    pub work_dir: PathBuf,
    pub sync_dir: PathBuf,
    pub dotfiles_dir: PathBuf,
    pub config_file: PathBuf,
    pub platform: PlatformType,
}

impl Settings {
    pub fn load(config_file: Option<PathBuf>) -> Result<Self> {
        let home_dir = home_dir()?;
        Ok(Self::for_home_with_config(home_dir, config_file))
    }

    pub fn for_home(home_dir: impl Into<PathBuf>) -> Self {
        Self::for_home_with_config(home_dir.into(), None)
    }

    pub fn for_home_with_config(home_dir: PathBuf, config_file: Option<PathBuf>) -> Self {
        let work_dir = home_dir.join(".config").join("df");
        let sync_dir = work_dir.join("dfctx");
        let dotfiles_dir = sync_dir.join(".dotfiles");
        let config_file = config_file.unwrap_or_else(|| sync_dir.join("config.json"));
        Self {
            home_dir,
            work_dir,
            sync_dir,
            dotfiles_dir,
            config_file,
            platform: PlatformType::current(),
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.work_dir).map_err(|err| io_error(&self.work_dir, err))?;
        std::fs::create_dir_all(&self.sync_dir).map_err(|err| io_error(&self.sync_dir, err))?;
        Ok(())
    }

    pub fn status_dir(&self) -> PathBuf {
        self.work_dir.join("cache").join(".status")
    }

    pub fn status_file(&self) -> PathBuf {
        self.status_dir().join("task_status.json")
    }

    pub fn mac_optimization_status_file(&self) -> PathBuf {
        self.status_dir().join("mac_optimization_status.json")
    }
}

pub fn expand_home(path: &str, home_dir: &Path) -> PathBuf {
    if path == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir.join(rest);
    }
    if let Some(rest) = path.strip_prefix("${HOME}") {
        return join_home_prefix(home_dir, rest);
    }
    if let Some(rest) = path.strip_prefix("$HOME") {
        return join_home_prefix(home_dir, rest);
    }
    PathBuf::from(path)
}

fn join_home_prefix(home_dir: &Path, rest: &str) -> PathBuf {
    let rest = rest.strip_prefix(std::path::MAIN_SEPARATOR).unwrap_or(rest);
    home_dir.join(rest)
}

fn home_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        return Ok(home);
    }
    if let Some(profile) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        return Ok(profile);
    }
    Err(message("无法解析 HOME 目录"))
}
