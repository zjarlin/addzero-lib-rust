use std::{env, fs, path::PathBuf};

use anyhow::{Context, anyhow};

pub fn data_dir_path() -> anyhow::Result<PathBuf> {
    let data_dir = xdg_dir("XDG_DATA_HOME", ".local/share")?;
    fs::create_dir_all(&data_dir)
        .with_context(|| format!("create data dir: {}", data_dir.display()))?;
    Ok(data_dir)
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
            return Ok(path.join("aio").join("software-center"));
        }
    }

    Ok(home_dir()?
        .join(fallback)
        .join("aio")
        .join("software-center"))
}
