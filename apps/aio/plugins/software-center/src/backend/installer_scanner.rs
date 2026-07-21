use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};

use crate::backend::installer_scanner_utils as utils;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerPackage {
    pub id: String,
    pub file_name: String,
    pub source_path: String,
    pub version: String,
    pub platform: String,
    pub arch: String,
    pub target_path: String,
    pub install_status: String,
    pub status: String,
    pub md5: String,
}

pub fn scan_installers() -> anyhow::Result<Vec<InstallerPackage>> {
    scan_installers_in(&utils::home_dir()?)
}

pub(crate) fn scan_installers_in(home: &Path) -> anyhow::Result<Vec<InstallerPackage>> {
    let candidates: [PathBuf; 2] = [home.join("Downloads"), home.join("Desktop")];
    let mut packages = Vec::new();

    for dir in candidates {
        scan_dir(&dir, 2, &mut packages)?;
    }

    packages.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(packages)
}

pub fn organize_installers() -> anyhow::Result<Vec<InstallerPackage>> {
    let packages = scan_installers()?;
    for package in &packages {
        archive_installer(package)?;
    }

    Ok(packages
        .into_iter()
        .map(|package| InstallerPackage {
            status: "archived".to_string(),
            ..package
        })
        .collect())
}

fn scan_dir(path: &Path, depth: usize, packages: &mut Vec<InstallerPackage>) -> anyhow::Result<()> {
    if depth == 0 || !path.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(path).with_context(|| format!("read dir: {}", path.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, depth - 1, packages)?;
        } else if utils::is_installer(&path) {
            packages.push(package_from_path(&path)?);
        }
    }

    Ok(())
}

fn package_from_path(path: &Path) -> anyhow::Result<InstallerPackage> {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| anyhow!("invalid installer filename: {}", path.display()))?;
    let platform = utils::detect_platform(&file_name);
    let arch = utils::detect_arch(&file_name);
    let version = utils::detect_version(&file_name).unwrap_or_else(|| "unknown".to_string());
    let target_path = utils::archive_target(&platform, &file_name)?;
    let md5 = utils::md5_file(path)?;

    Ok(InstallerPackage {
        id: format!("installer-{md5}"),
        file_name: file_name.clone(),
        source_path: path.to_string_lossy().into_owned(),
        version,
        platform,
        arch,
        target_path: target_path.to_string_lossy().into_owned(),
        install_status: utils::installed_status(&file_name),
        status: if target_path.exists() {
            "archived"
        } else {
            "pending"
        }
        .to_string(),
        md5,
    })
}

fn archive_installer(package: &InstallerPackage) -> anyhow::Result<()> {
    let source = Path::new(&package.source_path);
    let target = Path::new(&package.target_path);
    if target.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create installer archive dir: {}", parent.display()))?;
    }
    fs::hard_link(source, target)
        .or_else(|_| fs::copy(source, target).map(|_| ()))
        .with_context(|| {
            format!(
                "archive installer failed: {} -> {}",
                source.display(),
                target.display()
            )
        })
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::scan_installers_in;

    #[test]
    fn scans_candidate_directories_for_installers() {
        let root = env::temp_dir().join(format!(
            "software-center-scan-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("Downloads")).unwrap();
        fs::write(root.join("Downloads/Raycast-1.2.3-macOS-arm64.dmg"), "demo").unwrap();

        let scanned = scan_installers_in(&root).unwrap();

        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].platform, "macOS");
        fs::remove_dir_all(root).unwrap();
    }
}
