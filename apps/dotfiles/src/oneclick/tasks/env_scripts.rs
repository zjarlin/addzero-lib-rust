use crate::config::PlatformType;
use crate::error::Result;
use crate::settings::Settings;

use super::super::confirm::confirm_and_run;

pub(crate) fn run(settings: &Settings, assume_yes: bool, dry_run: bool) -> Result<()> {
    let command = match settings.platform {
        PlatformType::Windows => {
            r#"if (!(Test-Path $profile)) { New-Item -Path $profile -ItemType File -Force | Out-Null }; $block = @'
function setenv($name, $value) {
    Set-Item -Path "Env:$name" -Value $value
    [System.Environment]::SetEnvironmentVariable($name, $value, [System.EnvironmentVariableTarget]::User)
    Write-Host "Environment variable $name set to $value (applies to new sessions)" -ForegroundColor Green
}
'@; if ((Get-Content $profile -Raw) -notmatch 'function setenv') { Add-Content -Path $profile -Value $block }"#
        }
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => {
            "profile=\"$HOME/.zshrc\"; [ -n \"$BASH_VERSION\" ] && profile=\"$HOME/.bashrc\"; touch \"$profile\"; grep -q '^setenv()' \"$profile\" 2>/dev/null || cat >> \"$profile\" <<'EOF'\n\nsetenv() {\n    local name=\"$1\"\n    local value=\"$2\"\n    export \"$name=$value\"\n    local profile_file=\"\"\n    if [ -f \"$HOME/.bashrc\" ]; then\n        profile_file=\"$HOME/.bashrc\"\n    elif [ -f \"$HOME/.zshrc\" ]; then\n        profile_file=\"$HOME/.zshrc\"\n    fi\n    if [ -n \"$profile_file\" ]; then\n        printf 'export %s=\"%s\"\\n' \"$name\" \"$value\" >> \"$profile_file\"\n        echo \"Environment variable $name set to $value (added to $profile_file)\"\n    else\n        echo \"Environment variable $name set to $value (applies to current session only)\"\n    fi\n}\nEOF"
        }
    };
    confirm_and_run(assume_yes, dry_run, "写入 dotfiles 环境变量脚本", command)
}
