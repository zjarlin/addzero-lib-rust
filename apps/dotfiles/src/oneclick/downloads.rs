use crate::config::PlatformType;

use super::model::{ArchFamily, Download, arch_family};

pub(crate) fn finalshell_download() -> Option<Download> {
    match (PlatformType::current(), arch_family()) {
        (PlatformType::Windows, _) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_windows_x64.exe",
            "finalshell_windows_x64.exe",
        )),
        (PlatformType::Macos, ArchFamily::Arm) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_macos_arm64.pkg",
            "finalshell_macos_arm64.pkg",
        )),
        (PlatformType::Macos, _) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_macos_x64.pkg",
            "finalshell_macos_x64.pkg",
        )),
        (PlatformType::Linux, ArchFamily::Arm) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_linux_arm64.deb",
            "finalshell_linux_arm64.deb",
        )),
        (PlatformType::Linux, ArchFamily::Loong) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_linux_loong64.deb",
            "finalshell_linux_loong64.deb",
        )),
        (PlatformType::Linux, _) => Some(Download::new(
            "https://dl.hostbuf.com/finalshell3/finalshell_linux_x64.deb",
            "finalshell_linux_x64.deb",
        )),
        _ => None,
    }
}

pub(crate) fn finalshell_install_command() -> &'static str {
    match PlatformType::current() {
        PlatformType::Macos => "open \"$p\"",
        PlatformType::Linux | PlatformType::Unknown => {
            "if command -v apt >/dev/null 2>&1; then sudo dpkg -i \"$p\"; elif command -v rpm >/dev/null 2>&1; then sudo rpm -i \"$p\"; else echo \"请手动安装: $p\"; fi"
        }
        PlatformType::Windows => "",
    }
}

pub(crate) fn graalvm_download(platform: PlatformType) -> Option<Download> {
    let os = match platform {
        PlatformType::Macos => "macos",
        PlatformType::Linux | PlatformType::Unknown => "linux",
        PlatformType::Windows => "windows",
    };
    let arch = match arch_family() {
        ArchFamily::Arm => "aarch64",
        ArchFamily::X64 | ArchFamily::Loong => "x64",
    };
    let ext = if platform == PlatformType::Windows {
        "zip"
    } else {
        "tar.gz"
    };
    let filename = format!("graalvm-jdk-21_{os}-{arch}_bin.{ext}");
    Some(Download {
        url: format!("https://download.oracle.com/graalvm/21/latest/{filename}"),
        filename,
        version: None,
    })
}

pub(crate) fn zulu_downloads(platform: PlatformType) -> Vec<Download> {
    let suffix = match (platform, arch_family()) {
        (PlatformType::Macos, ArchFamily::Arm) => "macosx_aarch64.tar.gz",
        (PlatformType::Macos, _) => "macosx_x64.tar.gz",
        (PlatformType::Linux | PlatformType::Unknown, ArchFamily::Arm) => "linux_aarch64.tar.gz",
        (PlatformType::Linux | PlatformType::Unknown, _) => "linux_x64.tar.gz",
        (PlatformType::Windows, ArchFamily::Arm) => "win_aarch64.zip",
        (PlatformType::Windows, _) => "win_x64.zip",
    };

    [
        (8, "zulu8.76.0.17-ca-jdk8.0.402"),
        (17, "zulu17.48.15-ca-jdk17.0.10"),
        (21, "zulu21.36.17-ca-jdk21.0.6"),
    ]
    .into_iter()
    .map(|(version, stem)| {
        let filename = format!("{stem}-{suffix}");
        Download::versioned(
            version,
            format!("https://cdn.azul.com/zulu/bin/{filename}"),
            filename,
        )
    })
    .collect()
}

pub(crate) fn quark_download(platform: PlatformType) -> Option<Download> {
    match platform {
        PlatformType::Macos => Some(Download::new(
            "https://pc-download.quark.cn/download/37214/quarkmac/pcquark@default/QuarkMac_V4.6.0.558_mac_pf30004_(zh-cn)_release_(Build2491742).dmg?response-content-disposition=attachment;%20filename=%22Quark_V4.6.0.558%40%40%40dapi-7c1614e4-dd79-45a9-9d2e-45a664bcefeb%40%40%40.dmg%22;filename*=UTF-8%27%27Quark_V4.6.0.558%40%40%40dapi-7c1614e4-dd79-45a9-9d2e-45a664bcefeb%40%40%40.dmg",
            "QuarkMac_V4.6.0.558.dmg",
        )),
        PlatformType::Windows => Some(Download::new(
            "https://pc-download.quark.cn/download/37214/quarkwin/pcquark@default/QuarkWin_V4.6.0.558_win_pf30004_(zh-cn)_release_(Build2491742).exe",
            "QuarkWin_V4.6.0.558.exe",
        )),
        PlatformType::Linux | PlatformType::Unknown => None,
    }
}
