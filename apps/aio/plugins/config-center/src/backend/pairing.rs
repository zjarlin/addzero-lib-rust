use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, anyhow, bail};

use crate::backend::paths::config_center_config_dir_path;

const DEVICE_INFO_FILE_NAME: &str = "device-info.json";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingLocalInfo {
    pub device_name: String,
    pub fingerprint: String,
    pub home_path: String,
    pub metadata_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingRemoteInfo {
    pub device_name: String,
    pub fingerprint: String,
    pub home_path: String,
    pub metadata_path: String,
    pub exported_at: String,
    pub is_self: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingRemoteReadRequest {
    pub home_path: String,
    pub local_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredPairingDeviceInfo {
    device_name: String,
    fingerprint: String,
    exported_at: String,
}

pub fn ensure_local_pairing_device_info() -> anyhow::Result<()> {
    let _ = load_or_create_local_device_info()?;
    Ok(())
}

pub fn local_pairing_info() -> anyhow::Result<PairingLocalInfo> {
    let home_path = canonical_home_dir()?;
    let metadata_path = local_metadata_path()?;
    let stored = load_or_create_local_device_info()?;
    Ok(PairingLocalInfo {
        device_name: stored.device_name,
        fingerprint: stored.fingerprint,
        home_path: home_path.to_string_lossy().into_owned(),
        metadata_path: metadata_path.to_string_lossy().into_owned(),
    })
}

pub fn read_remote_pairing_info(
    request: PairingRemoteReadRequest,
) -> anyhow::Result<PairingRemoteInfo> {
    let home_path = canonicalize_home_path(&request.home_path)?;
    let metadata_path = remote_metadata_path(&home_path);
    let stored = read_stored_device_info(&metadata_path).with_context(|| {
        format!(
            "remote device info missing; expected {}",
            metadata_path.display()
        )
    })?;
    let local_home = canonical_home_dir()?;
    let is_self = stored.fingerprint == request.local_fingerprint || home_path == local_home;

    Ok(PairingRemoteInfo {
        device_name: stored.device_name,
        fingerprint: stored.fingerprint,
        home_path: home_path.to_string_lossy().into_owned(),
        metadata_path: metadata_path.to_string_lossy().into_owned(),
        exported_at: stored.exported_at,
        is_self,
    })
}

fn load_or_create_local_device_info() -> anyhow::Result<StoredPairingDeviceInfo> {
    let metadata_path = local_metadata_path()?;
    if metadata_path.exists() {
        return read_stored_device_info(&metadata_path).or_else(|_| {
            let next = new_stored_device_info()?;
            write_stored_device_info(&metadata_path, &next)?;
            Ok(next)
        });
    }

    let next = new_stored_device_info()?;
    write_stored_device_info(&metadata_path, &next)?;
    Ok(next)
}

fn read_stored_device_info(path: &Path) -> anyhow::Result<StoredPairingDeviceInfo> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read device identity: {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse device identity: {}", path.display()))
}

fn write_stored_device_info(path: &Path, stored: &StoredPairingDeviceInfo) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create pairing metadata dir: {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(stored)?;
    fs::write(path, text).with_context(|| format!("write device identity: {}", path.display()))
}

fn new_stored_device_info() -> anyhow::Result<StoredPairingDeviceInfo> {
    let home_path = canonical_home_dir()?;
    let device_name = default_device_name(&home_path);
    Ok(StoredPairingDeviceInfo {
        device_name,
        fingerprint: build_fingerprint(&home_path)?,
        exported_at: now_text(),
    })
}

fn build_fingerprint(home_path: &Path) -> anyhow::Result<String> {
    let mut hasher = DefaultHasher::new();
    home_path.hash(&mut hasher);
    env::var("USER").unwrap_or_default().hash(&mut hasher);
    env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .unwrap_or_default()
        .hash(&mut hasher);
    current_epoch_nanos().hash(&mut hasher);
    let hex = format!("{:016X}", hasher.finish());
    Ok(format!(
        "ed25519:{}:{}:{}:{}",
        &hex[0..4],
        &hex[4..8],
        &hex[8..12],
        &hex[12..16]
    ))
}

fn default_device_name(home_path: &Path) -> String {
    env::var("HOSTNAME")
        .or_else(|_| env::var("COMPUTERNAME"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            home_path
                .file_name()
                .and_then(|value| value.to_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("current-machine")
                .to_string()
        })
}

fn local_metadata_path() -> anyhow::Result<PathBuf> {
    Ok(config_center_config_dir_path()?.join(DEVICE_INFO_FILE_NAME))
}

fn remote_metadata_path(home_path: &Path) -> PathBuf {
    home_path
        .join(".config")
        .join("aio")
        .join("config-center")
        .join(DEVICE_INFO_FILE_NAME)
}

fn canonical_home_dir() -> anyhow::Result<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot resolve current home dir"))?;
    home.canonicalize()
        .with_context(|| format!("canonicalize home dir: {}", home.display()))
}

fn canonicalize_home_path(raw_path: &str) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(raw_path.trim());
    if raw_path.trim().is_empty() {
        bail!("remote home path is empty");
    }
    if !path.is_absolute() {
        bail!("remote home path must be absolute");
    }
    if !path.exists() {
        let message = format!("remote home path does not exist: {}", path.display());
        bail!(message);
    }
    if !path.is_dir() {
        let message = format!("remote home path is not a directory: {}", path.display());
        bail!(message);
    }
    path.canonicalize()
        .with_context(|| format!("canonicalize remote home dir: {}", path.display()))
}

fn now_text() -> String {
    format!("unix:{}", current_epoch_secs())
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn current_epoch_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::{PairingRemoteReadRequest, read_remote_pairing_info};
    use std::{
        env, fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn read_remote_pairing_info_marks_self_when_fingerprint_matches() {
        let root = new_remote_home("ThinkPad", "ed25519:AAAA:BBBB:CCCC:DDDD");

        let remote = read_remote_pairing_info(PairingRemoteReadRequest {
            home_path: root.to_string_lossy().into_owned(),
            local_fingerprint: "ed25519:AAAA:BBBB:CCCC:DDDD".to_string(),
        })
        .unwrap();

        assert!(remote.is_self);
        fs::remove_dir_all(root).unwrap();
    }

    fn new_remote_home(device_name: &str, fingerprint: &str) -> std::path::PathBuf {
        let root = env::temp_dir().join(format!(
            "config-center-pair-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let metadata_path = root
            .join(".config")
            .join("aio")
            .join("config-center")
            .join("device-info.json");
        fs::create_dir_all(metadata_path.parent().unwrap()).unwrap();
        fs::write(
            &metadata_path,
            format!(
                "{{\"deviceName\":\"{device_name}\",\"fingerprint\":\"{fingerprint}\",\"exportedAt\":\"unix:1\"}}"
            ),
        )
        .unwrap();
        assert!(Path::new(&metadata_path).exists());
        root
    }
}
