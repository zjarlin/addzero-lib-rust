use crate::backend::installer_scanner::InstallerPackage;

/// 判断扫描到的安装包是否能匹配软件目录中的 slug 或标题。
pub fn installer_matches_catalog(package: &InstallerPackage, slug: &str, title: &str) -> bool {
    let normalized_name = package.file_name.to_ascii_lowercase();
    normalized_name.contains(&slug.to_ascii_lowercase())
        || normalized_name.contains(&title.to_ascii_lowercase().replace(' ', ""))
        || normalized_name.contains(&title.to_ascii_lowercase().replace(' ', "-"))
}

#[cfg(test)]
mod tests {
    use crate::backend::catalog_match::installer_matches_catalog;
    use crate::backend::installer_scanner::InstallerPackage;

    #[test]
    fn links_installers_to_catalog_slugs() {
        let package = InstallerPackage {
            id: "1".to_string(),
            file_name: "raycast-1.2.3-macos-arm64.dmg".to_string(),
            source_path: String::new(),
            version: "1.2.3".to_string(),
            platform: "macOS".to_string(),
            arch: "arm64".to_string(),
            target_path: String::new(),
            install_status: "unconfirmed".to_string(),
            status: "pending".to_string(),
            md5: "x".to_string(),
        };

        assert!(installer_matches_catalog(&package, "raycast", "Raycast"));
    }
}
