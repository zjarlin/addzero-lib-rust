use crate::config::{PlatformConfig, PlatformType};
use crate::error::{Result, message};
use crate::platform::{command_exists, quote_sh, run_shell, run_shell_checked};

#[derive(Clone, Debug)]
pub enum PackageManager {
    Homebrew,
    Apt,
    Winget,
    Chocolatey,
}

impl PackageManager {
    pub fn from_config(platform: PlatformType, config: &PlatformConfig) -> Result<Self> {
        if let Some(name) = &config.package_manager {
            if let Some(manager) = Self::from_name(name) {
                return Ok(manager);
            }
        }

        match platform {
            PlatformType::Macos => Ok(Self::Homebrew),
            PlatformType::Linux | PlatformType::Unknown => Ok(Self::Apt),
            PlatformType::Windows => {
                if command_exists("winget") {
                    Ok(Self::Winget)
                } else {
                    Ok(Self::Chocolatey)
                }
            }
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Homebrew => "Homebrew",
            Self::Apt => "APT",
            Self::Winget => "Winget",
            Self::Chocolatey => "Chocolatey",
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            Self::Homebrew => command_exists("brew"),
            Self::Apt => command_exists("apt"),
            Self::Winget => command_exists("winget"),
            Self::Chocolatey => command_exists("choco"),
        }
    }

    pub fn install_self(&self) -> Result<()> {
        if self.is_available() {
            return Ok(());
        }

        match self {
            Self::Homebrew => run_shell_checked(
                r#"/bin/zsh -c "$(curl -fsSL https://gitee.com/cunkai/HomebrewCN/raw/master/Homebrew.sh)""#,
            )
            .map(|_| ()),
            Self::Apt => Err(message("APT 不可用，请先在 Debian/Ubuntu 系统安装 apt")),
            Self::Winget => Err(message(
                "Winget 不可用，请从 Microsoft Store 安装 App Installer 或升级 Windows",
            )),
            Self::Chocolatey => run_shell_checked(
                "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))",
            )
            .map(|_| ()),
        }
    }

    pub fn update_index(&self) -> Result<()> {
        match self {
            Self::Homebrew => run_shell_checked("brew update").map(|_| ()),
            Self::Apt => run_shell_checked("sudo apt update").map(|_| ()),
            Self::Winget => Ok(()),
            Self::Chocolatey => run_shell_checked("choco upgrade chocolatey -y").map(|_| ()),
        }
    }

    pub fn install(&self, package: &str) -> Result<()> {
        let package = quote_sh(package);
        let command = match self {
            Self::Homebrew => format!("brew install {package}"),
            Self::Apt => format!("sudo apt install -y {package}"),
            Self::Winget => format!(
                "winget install --silent --accept-package-agreements --accept-source-agreements {package}"
            ),
            Self::Chocolatey => format!("choco install {package} -y"),
        };
        run_shell_checked(&command).map(|_| ())
    }

    pub fn uninstall(&self, package: &str) -> Result<()> {
        let package = quote_sh(package);
        let command = match self {
            Self::Homebrew => format!("brew uninstall {package}"),
            Self::Apt => format!("sudo apt remove -y {package}"),
            Self::Winget => format!("winget uninstall --silent {package}"),
            Self::Chocolatey => format!("choco uninstall {package} -y"),
        };
        run_shell_checked(&command).map(|_| ())
    }

    pub fn is_installed(&self, package: &str) -> bool {
        let package = quote_sh(package);
        let command = match self {
            Self::Homebrew => {
                format!("brew list --formula | grep -qx {package} || command -v {package}")
            }
            Self::Apt => format!("dpkg -l | grep -w {package}"),
            Self::Winget => format!("winget list --exact --query {package}"),
            Self::Chocolatey => format!("choco list --local-only {package}"),
        };
        run_shell(&command).is_ok_and(|output| output.success())
    }

    pub fn version(&self, package: &str) -> Result<Option<String>> {
        if !self.is_installed(package) {
            return Ok(None);
        }

        let package = quote_sh(package);
        let command = match self {
            Self::Homebrew => format!(
                "brew info --json=v1 {package} | grep '\"version\"' | head -n 1 | awk -F ':' '{{print $2}}' | sed 's/[\",]//g' | tr -d '[[:space:]]'"
            ),
            Self::Apt => format!("dpkg -s {package} | grep Version | cut -d ' ' -f 2"),
            Self::Winget => {
                format!("winget show --exact --query {package} | grep -i '^Version:' | head -n 1")
            }
            Self::Chocolatey => format!("choco list --local-only {package} | head -n 1"),
        };

        let output = run_shell(&command)?;
        if output.success() && !output.stdout.trim().is_empty() {
            Ok(Some(output.stdout.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, keyword: &str) -> Result<Vec<String>> {
        let keyword = quote_sh(keyword);
        let command = match self {
            Self::Homebrew => format!("brew search {keyword}"),
            Self::Apt => format!("apt search {keyword} 2>/dev/null | awk -F/ '/\\// {{print $1}}'"),
            Self::Winget => format!("winget search {keyword}"),
            Self::Chocolatey => format!("choco search {keyword}"),
        };

        let output = run_shell(&command)?;
        if !output.success() {
            return Ok(Vec::new());
        }

        Ok(output
            .stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect())
    }

    fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "brew" | "homebrew" => Some(Self::Homebrew),
            "apt" => Some(Self::Apt),
            "winget" => Some(Self::Winget),
            "choco" | "chocolatey" => Some(Self::Chocolatey),
            _ => None,
        }
    }
}
