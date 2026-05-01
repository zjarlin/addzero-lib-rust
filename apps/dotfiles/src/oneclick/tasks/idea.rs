use std::path::PathBuf;

use crate::config::PlatformType;
use crate::error::Result;
use crate::platform::quote_sh;
use crate::settings::Settings;

use super::super::confirm::confirm_and_run;
use super::super::shell::quote_ps_path;

pub(crate) fn run(settings: &Settings, assume_yes: bool, dry_run: bool) -> Result<()> {
    let Some(path) = find_idea_properties(settings) else {
        println!("未找到 IntelliJ IDEA idea.properties，跳过配置");
        return Ok(());
    };

    let path_display = path.display().to_string();
    let command = match settings.platform {
        PlatformType::Windows => format!(
            "$p = {}; $lines = Get-Content $p -ErrorAction Stop; $lines = $lines | Where-Object {{ $_ -notmatch '^#?idea\\.(config|plugins)\\.path=' }}; $lines += 'idea.config.path=${{user.home}}/.config/IntelliJIdea/config'; $lines += 'idea.plugins.path=${{idea.config.path}}/plugins'; Set-Content -Path $p -Value $lines",
            quote_ps_path(&path)
        ),
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => format!(
            "p={}; grep -vE '^#?idea\\.(config|plugins)\\.path=' \"$p\" > \"$p.tmp\" && printf '%s\\n%s\\n' 'idea.config.path=${{user.home}}/.config/IntelliJIdea/config' 'idea.plugins.path=${{idea.config.path}}/plugins' >> \"$p.tmp\" && mv \"$p.tmp\" \"$p\"",
            quote_sh(&path_display)
        ),
    };
    confirm_and_run(assume_yes, dry_run, "配置 IntelliJ IDEA XDG 路径", &command)
}

fn find_idea_properties(settings: &Settings) -> Option<PathBuf> {
    let home = settings.home_dir.to_string_lossy();
    let candidates = match settings.platform {
        PlatformType::Macos => vec![
            format!("{home}/Applications/IntelliJ IDEA Ultimate.app/Contents/bin/idea.properties"),
            format!("{home}/Applications/IntelliJ IDEA.app/Contents/bin/idea.properties"),
            "/Applications/IntelliJ IDEA.app/Contents/bin/idea.properties".to_string(),
            "/Applications/IntelliJ IDEA Ultimate.app/Contents/bin/idea.properties".to_string(),
        ],
        PlatformType::Windows => {
            let program_files =
                std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
            let program_files_x86 = std::env::var("ProgramFiles(x86)")
                .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
            vec![
                format!("{program_files}\\JetBrains\\IntelliJ IDEA\\bin\\idea.properties"),
                format!("{program_files_x86}\\JetBrains\\IntelliJ IDEA\\bin\\idea.properties"),
                format!(
                    "{program_files}\\JetBrains\\IntelliJ IDEA Community Edition\\bin\\idea.properties"
                ),
                format!(
                    "{program_files_x86}\\JetBrains\\IntelliJ IDEA Community Edition\\bin\\idea.properties"
                ),
                format!("{program_files}\\JetBrains\\IntelliJ IDEA Ultimate\\bin\\idea.properties"),
                format!(
                    "{program_files_x86}\\JetBrains\\IntelliJ IDEA Ultimate\\bin\\idea.properties"
                ),
            ]
        }
        PlatformType::Linux | PlatformType::Unknown => Vec::new(),
    };

    candidates
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
}
