use std::path::Path;

use crate::config::PlatformType;
use crate::error::Result;
use crate::platform::quote_sh;
use crate::settings::Settings;

use super::super::confirm::confirm_and_run;
use super::super::downloads::zulu_downloads;
use super::super::model::Download;
use super::super::shell::{quote_ps, quote_ps_path};
use super::useful::useful_soft_dir;

pub(crate) fn run(settings: &Settings, assume_yes: bool, dry_run: bool) -> Result<()> {
    let downloads = zulu_downloads(settings.platform);
    if downloads.is_empty() {
        println!("当前平台不支持 Zulu JDK 自动下载");
        return Ok(());
    }

    let useful_dir = useful_soft_dir(settings);
    let command = match settings.platform {
        PlatformType::Windows => zulu_windows_command(&useful_dir, &downloads),
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => {
            zulu_unix_command(&useful_dir, &downloads)
        }
    };
    confirm_and_run(
        assume_yes,
        dry_run,
        "下载 Zulu JDK 8/17/21 并配置 JAVA_HOME",
        &command,
    )
}

fn zulu_windows_command(useful_dir: &Path, downloads: &[Download]) -> String {
    let mut parts = vec![format!(
        "New-Item -ItemType Directory -Force -Path {} | Out-Null",
        quote_ps_path(useful_dir)
    )];
    for download in downloads {
        let archive = useful_dir.join(&download.filename);
        let extract_dir = useful_dir.join(format!("zulu{}", download.version.unwrap_or(0)));
        parts.push(format!(
            "if (!(Test-Path {})) {{ (New-Object Net.WebClient).DownloadFile({}, {}); Expand-Archive -Path {} -DestinationPath {} -Force }}",
            quote_ps_path(&extract_dir),
            quote_ps(&download.url),
            quote_ps_path(&archive),
            quote_ps_path(&archive),
            quote_ps_path(&extract_dir)
        ));
    }
    parts.push(format!(
        "$jdk = Get-ChildItem -Path {} -Directory -Recurse | Where-Object {{ Test-Path (Join-Path $_.FullName 'bin\\java.exe') }} | Select-Object -First 1; if ($jdk) {{ [Environment]::SetEnvironmentVariable('JAVA_HOME', $jdk.FullName, 'User'); [Environment]::SetEnvironmentVariable('PATH', '%JAVA_HOME%\\bin;' + [Environment]::GetEnvironmentVariable('PATH', 'User'), 'User') }}",
        quote_ps_path(useful_dir)
    ));
    parts.join("; ")
}

fn zulu_unix_command(useful_dir: &Path, downloads: &[Download]) -> String {
    let mut parts = vec![format!(
        "mkdir -p {}",
        quote_sh(&useful_dir.to_string_lossy())
    )];
    for download in downloads {
        let archive = useful_dir.join(&download.filename);
        let extract_dir = useful_dir.join(format!("zulu{}", download.version.unwrap_or(0)));
        parts.push(format!(
            "if [ ! -d {} ]; then mkdir -p {}; curl -s -L -o {} {}; tar -xzf {} -C {} --strip-components=1; rm -f {}; fi",
            quote_sh(&extract_dir.to_string_lossy()),
            quote_sh(&extract_dir.to_string_lossy()),
            quote_sh(&archive.to_string_lossy()),
            quote_sh(&download.url),
            quote_sh(&archive.to_string_lossy()),
            quote_sh(&extract_dir.to_string_lossy()),
            quote_sh(&archive.to_string_lossy())
        ));
    }
    let java_home = useful_dir.join("zulu17");
    parts.push(format!(
        "profile=\"$HOME/.zshrc\"; [ -n \"$BASH_VERSION\" ] && profile=\"$HOME/.bashrc\"; touch \"$profile\"; grep -q '^export JAVA_HOME=' \"$profile\" && sed -i.bak 's|^export JAVA_HOME=.*|export JAVA_HOME=\"{}\"|' \"$profile\" || printf '\\nexport JAVA_HOME=\"{}\"\\nexport PATH=\"$JAVA_HOME/bin:$PATH\"\\n' >> \"$profile\"",
        java_home.to_string_lossy(),
        java_home.to_string_lossy()
    ));
    parts.join("; ")
}
