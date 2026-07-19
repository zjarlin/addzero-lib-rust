use std::{
    env, fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};

const INSTALLER_EXTENSIONS: &[&str] =
    &["dmg", "pkg", "zip", "exe", "msi", "deb", "rpm", "appimage"];

pub(crate) fn archive_target(platform: &str, file_name: &str) -> anyhow::Result<PathBuf> {
    Ok(crate::backend::paths::data_dir_path()?
        .join("installers")
        .join(platform)
        .join(file_name))
}

pub(crate) fn is_installer(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .is_some_and(|ext| INSTALLER_EXTENSIONS.contains(&ext.as_str()))
}

pub(crate) fn detect_platform(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if ["mac", "darwin", "apple", "dmg", "pkg"]
        .iter()
        .any(|part| lower.contains(part))
    {
        return "macOS".to_string();
    }
    if ["win", "windows", "exe", "msi"]
        .iter()
        .any(|part| lower.contains(part))
    {
        return "Windows".to_string();
    }
    "Unix".to_string()
}

pub(crate) fn detect_arch(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if ["arm64", "aarch64", "apple"]
        .iter()
        .any(|part| lower.contains(part))
    {
        return "arm64".to_string();
    }
    if ["x86_64", "amd64", "x64"]
        .iter()
        .any(|part| lower.contains(part))
    {
        return "x86_64".to_string();
    }
    if lower.contains("universal") {
        return "universal".to_string();
    }
    "unknown".to_string()
}

pub(crate) fn detect_version(file_name: &str) -> Option<String> {
    let mut parts = file_name.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.'));
    parts
        .find(|part| {
            let mut digits = 0;
            let mut dots = 0;
            for ch in part.chars() {
                if ch.is_ascii_digit() {
                    digits += 1;
                }
                if ch == '.' {
                    dots += 1;
                }
            }
            digits > 0 && dots > 0
        })
        .map(str::to_string)
}

pub(crate) fn installed_status(file_name: &str) -> String {
    let app_name = file_name.split(['-', '_', '.']).next().unwrap_or(file_name);
    let app_path = Path::new("/Applications").join(format!("{app_name}.app"));
    if app_path.exists() {
        "installed".to_string()
    } else {
        "unconfirmed".to_string()
    }
}

pub(crate) fn md5_file(path: &Path) -> anyhow::Result<String> {
    let file = fs::File::open(path)
        .with_context(|| format!("open installer for md5: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut context = md5::Context::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader
            .read(&mut buffer)
            .with_context(|| format!("read installer for md5: {}", path.display()))?;
        if read == 0 {
            break;
        }
        context.consume(&buffer[..read]);
    }

    Ok(format!("{:x}", context.finalize()))
}

pub(crate) fn home_dir() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot resolve user home"))
}

#[cfg(test)]
mod tests {
    use super::{detect_arch, detect_platform, detect_version};

    #[test]
    fn detects_installer_properties_from_filename() {
        assert_eq!(detect_platform("Raycast-macOS-arm64.dmg"), "macOS");
        assert_eq!(detect_arch("Raycast-macOS-arm64.dmg"), "arm64");
        assert_eq!(
            detect_version("Raycast-1.2.3-macOS-arm64.dmg").as_deref(),
            Some("1.2.3")
        );
    }
}
