use crate::config::PlatformType;
use crate::error::Result;
use crate::platform::quote_sh;

use super::super::confirm::confirm_and_run;
use super::super::downloads::{finalshell_download, finalshell_install_command};
use super::super::shell::quote_ps;

pub(crate) fn run(assume_yes: bool, dry_run: bool) -> Result<()> {
    let Some(download) = finalshell_download() else {
        println!("当前平台不支持 FinalShell 自动下载");
        return Ok(());
    };

    let command = if PlatformType::current() == PlatformType::Windows {
        format!(
            "$p = Join-Path $env:TEMP {}; (New-Object Net.WebClient).DownloadFile({}, $p); Start-Process $p -Verb RunAs",
            quote_ps(&download.filename),
            quote_ps(&download.url)
        )
    } else {
        format!(
            "p=\"${{TMPDIR:-/tmp}}/{}\"; curl -L -o \"$p\" {}; {}",
            download.filename,
            quote_sh(&download.url),
            finalshell_install_command()
        )
    };
    confirm_and_run(
        assume_yes,
        dry_run,
        "下载并打开 FinalShell 安装器",
        &command,
    )
}
