use crate::config::PlatformType;
use crate::error::Result;
use crate::platform::quote_sh;
use crate::settings::Settings;

use super::super::confirm::confirm_and_run;
use super::super::downloads::graalvm_download;
use super::super::shell::{quote_ps, quote_ps_path};
use super::useful::useful_soft_dir;

pub(crate) fn run(settings: &Settings, assume_yes: bool, dry_run: bool) -> Result<()> {
    let useful_dir = useful_soft_dir(settings);
    let Some(download) = graalvm_download(settings.platform) else {
        println!("当前平台不支持 GraalVM 自动下载");
        return Ok(());
    };
    let destination = useful_dir.join(&download.filename);
    if destination.exists() {
        println!("GraalVM 已存在: {}", destination.display());
        return Ok(());
    }

    let command = match settings.platform {
        PlatformType::Windows => format!(
            "New-Item -ItemType Directory -Force -Path {} | Out-Null; (New-Object Net.WebClient).DownloadFile({}, {})",
            quote_ps_path(&useful_dir),
            quote_ps(&download.url),
            quote_ps_path(&destination)
        ),
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => format!(
            "mkdir -p {}; curl -L -o {} {}",
            quote_sh(&useful_dir.to_string_lossy()),
            quote_sh(&destination.to_string_lossy()),
            quote_sh(&download.url)
        ),
    };
    confirm_and_run(assume_yes, dry_run, "下载 GraalVM 到 useful 目录", &command)
}
