use std::{env, fs, path::PathBuf};

use anyhow::{Context, anyhow};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigCenterPaths {
    pub data_dir: String,
    pub config_dir: String,
    pub state_dir: String,
    pub cache_dir: String,
}

pub fn resolve_config_center_paths() -> anyhow::Result<ConfigCenterPaths> {
    let data_dir = xdg_dir("XDG_DATA_HOME", ".local/share")?;
    let config_dir = xdg_dir("XDG_CONFIG_HOME", ".config")?;
    let state_dir = xdg_dir("XDG_STATE_HOME", ".local/state")?;
    let cache_dir = xdg_dir("XDG_CACHE_HOME", ".cache")?;

    for path in [&data_dir, &config_dir, &state_dir, &cache_dir] {
        fs::create_dir_all(path).with_context(|| format!("create xdg dir: {}", path.display()))?;
    }

    Ok(ConfigCenterPaths {
        data_dir: path_string(data_dir),
        config_dir: path_string(config_dir),
        state_dir: path_string(state_dir),
        cache_dir: path_string(cache_dir),
    })
}

pub fn config_center_data_dir_path() -> anyhow::Result<PathBuf> {
    let data_dir = xdg_dir("XDG_DATA_HOME", ".local/share")?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("create xdg data dir: {}", data_dir.display()))?;
    Ok(data_dir)
}

pub fn config_center_config_dir_path() -> anyhow::Result<PathBuf> {
    let config_dir = xdg_dir("XDG_CONFIG_HOME", ".config")?;
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("create xdg config dir: {}", config_dir.display()))?;
    Ok(config_dir)
}

pub fn config_center_state_dir_path() -> anyhow::Result<PathBuf> {
    let state_dir = xdg_dir("XDG_STATE_HOME", ".local/state")?;
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("create xdg state dir: {}", state_dir.display()))?;
    Ok(state_dir)
}

fn home_dir() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot resolve user home"))
}

fn xdg_dir(env_key: &str, fallback: &str) -> anyhow::Result<PathBuf> {
    if let Some(value) = env::var_os(env_key) {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Ok(path.join("aio").join("config-center"));
        }
    }

    Ok(home_dir()?.join(fallback).join("aio").join("config-center"))
}

fn path_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
