use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::{Config, PlatformType};
use crate::error::{Result, io_error};
use crate::package_manager::PackageManager;
use crate::platform::{
    command_exists, prompt_yes_no, quote_sh, run_interactive_shell, run_shell, run_shell_checked,
};
use crate::settings::Settings;
use crate::status::StatusStore;

pub fn init_git(settings: &Settings, config: &Config, assume_yes: bool, force: bool) -> Result<()> {
    let marker = settings.status_dir().join("AutoInitGit.status");
    if marker.exists() && !force {
        println!("AutoInitGit 已经初始化过，跳过执行");
        return Ok(());
    }

    if !command_exists("git") {
        println!("未检测到 Git");
        let manager = PackageManager::from_config(
            settings.platform,
            config.current_platform_config(settings.platform),
        )?;
        let package = if settings.platform == PlatformType::Windows {
            "Git.Git"
        } else {
            "git"
        };
        if prompt_yes_no("是否安装 Git?", true, assume_yes)? {
            manager.install(package)?;
        }
    } else {
        println!("检测到 Git");
    }

    let username = run_shell("git config --global user.name")?.stdout;
    if username.trim().is_empty()
        && prompt_yes_no("Git 用户名未配置，是否现在配置?", true, assume_yes)?
    {
        println!("请用 `dotfiles git-config <username> <email>` 设置 Git 身份");
    }

    let email = run_shell("git config --global user.email")?.stdout;
    if email.trim().is_empty() && prompt_yes_no("Git 邮箱未配置，是否现在配置?", true, assume_yes)?
    {
        println!("请用 `dotfiles git-config <username> <email>` 设置 Git 身份");
    }

    write_marker(settings, "AutoInitGit")?;
    StatusStore::new(settings.status_file()).record_completed("AutoInitGit", "Git 初始化")?;
    Ok(())
}

pub fn init_node(
    settings: &Settings,
    config: &Config,
    assume_yes: bool,
    force: bool,
) -> Result<()> {
    let marker = settings.status_dir().join("AutoInitNodejs.status");
    if marker.exists() && !force {
        println!("AutoInitNodejs 已经初始化过，跳过执行");
        return Ok(());
    }

    if !command_exists("node") {
        println!("Node.js 未安装");
        if prompt_yes_no("是否尝试通过当前包管理器安装 Node.js?", true, assume_yes)? {
            let manager = PackageManager::from_config(
                settings.platform,
                config.current_platform_config(settings.platform),
            )?;
            manager.install("node")?;
        }
    } else {
        println!("Node.js 已安装");
        let installed = ["npm", "pnpm", "yarn"]
            .into_iter()
            .filter(|name| command_exists(name))
            .collect::<Vec<_>>();
        if installed.is_empty()
            && prompt_yes_no("未检测到 npm/pnpm/yarn，是否安装 pnpm?", true, assume_yes)?
        {
            run_shell_checked("npm install -g pnpm")?;
        } else if !installed.is_empty() {
            println!("已安装的包管理器: {}", installed.join(", "));
        }
    }

    write_marker(settings, "AutoInitNodejs")?;
    StatusStore::new(settings.status_file())
        .record_completed("AutoInitNodejs", "Node.js 初始化")?;
    Ok(())
}

pub fn init_packages(
    settings: &Settings,
    config: &Config,
    packages: &[String],
    assume_yes: bool,
) -> Result<()> {
    let manager = PackageManager::from_config(
        settings.platform,
        config.current_platform_config(settings.platform),
    )?;
    println!("使用包管理器: {}", manager.name());

    if !manager.is_available() {
        println!("{} 不可用，尝试安装...", manager.name());
        manager.install_self()?;
    }

    let missing = packages
        .iter()
        .filter(|package| !manager.is_installed(package))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        println!("配置中的软件包均已安装");
        return Ok(());
    }

    println!("这些安装包还没有安装过: {}", missing.join(", "));
    if !prompt_yes_no("是否确认安装?", false, assume_yes)? {
        println!("已跳过安装");
        return Ok(());
    }

    manager.update_index()?;
    let mut success = Vec::new();
    for package in &missing {
        match manager.install(package) {
            Ok(()) => success.push(package.clone()),
            Err(err) => eprintln!("安装 {package} 失败: {err}"),
        }
    }
    println!(
        "安装完成: {}/{} 个软件包成功安装",
        success.len(),
        missing.len()
    );
    Ok(())
}

pub fn init_macos(settings: &Settings, assume_yes: bool) -> Result<()> {
    if settings.platform != PlatformType::Macos {
        println!("当前不是 macOS，跳过 macOS 优化");
        return Ok(());
    }

    let mut status = load_macos_status(settings)?;
    let optimizations = macos_optimizations();
    let pending = optimizations
        .iter()
        .filter(|item| !status.executed_hashes.contains(&hash_command(item.command)))
        .collect::<Vec<_>>();

    if pending.is_empty() {
        println!("所有 macOS 优化项均已执行，无需重复执行");
        return Ok(());
    }

    println!("推荐的 macOS 系统优化项:");
    for (index, item) in pending.iter().enumerate() {
        println!("{}. {}", index + 1, item.description);
    }

    for item in pending {
        if prompt_yes_no(
            &format!("应用优化项 [{}]?", item.description),
            true,
            assume_yes,
        )? {
            run_shell_checked(item.command)?;
            status.executed_hashes.insert(hash_command(item.command));
            save_macos_status(settings, &status)?;
        }
    }

    let _ = run_shell("killall Finder");
    let _ = run_shell("killall Dock");
    Ok(())
}

pub fn init_homebrew() -> Result<()> {
    if PlatformType::current() != PlatformType::Macos {
        println!("当前不是 macOS，跳过 Homebrew 初始化");
        return Ok(());
    }
    if command_exists("brew") {
        println!("Homebrew 已安装");
        return Ok(());
    }
    run_interactive_shell(
        r#"/bin/zsh -c "$(curl -fsSL https://gitee.com/cunkai/HomebrewCN/raw/master/Homebrew.sh)""#,
    )
}

pub fn init_ohmyzsh() -> Result<()> {
    if PlatformType::current() != PlatformType::Macos {
        println!("当前不是 macOS，跳过 Oh My Zsh 初始化");
        return Ok(());
    }
    if !command_exists("zsh") {
        println!("未检测到 zsh");
        return Ok(());
    }
    if !std::path::Path::new(&format!(
        "{}/.oh-my-zsh",
        std::env::var("HOME").unwrap_or_default()
    ))
    .exists()
    {
        run_shell_checked(
            r#"sh -c "$(curl -fsSL https://gitee.com/allenjia09/ohmyzsh/raw/master/tools/install.sh)""#,
        )?;
    }
    let _ = run_shell("brew install zsh-autosuggestions");
    let _ = run_shell("brew install zsh-syntax-highlighting");
    Ok(())
}

pub fn init_docker() -> Result<()> {
    run_interactive_shell("bash <(curl -sSL https://linuxmirrors.cn/docker.sh)")
}

pub fn init_lazyvim() -> Result<()> {
    match PlatformType::current() {
        PlatformType::Windows => {
            run_shell(
                "Move-Item $env:LOCALAPPDATA\\nvim $env:LOCALAPPDATA\\nvim.bak -ErrorAction SilentlyContinue",
            )?;
            run_shell(
                "Move-Item $env:LOCALAPPDATA\\nvim-data $env:LOCALAPPDATA\\nvim-data.bak -ErrorAction SilentlyContinue",
            )?;
            run_shell_checked(
                "git clone https://github.com/LazyVim/starter $env:LOCALAPPDATA\\nvim",
            )?;
            run_shell(
                "Remove-Item $env:LOCALAPPDATA\\nvim\\.git -Recurse -Force -ErrorAction SilentlyContinue",
            )?;
        }
        PlatformType::Macos | PlatformType::Linux | PlatformType::Unknown => {
            run_shell("mv ~/.config/nvim ~/.config/nvim.bak 2>/dev/null || true")?;
            run_shell("mv ~/.local/share/nvim ~/.local/share/nvim.bak 2>/dev/null || true")?;
            run_shell("mv ~/.local/state/nvim ~/.local/state/nvim.bak 2>/dev/null || true")?;
            run_shell("mv ~/.cache/nvim ~/.cache/nvim.bak 2>/dev/null || true")?;
            run_shell_checked("git clone https://github.com/LazyVim/starter ~/.config/nvim")?;
            run_shell_checked("rm -rf ~/.config/nvim/.git")?;
        }
    }
    println!("LazyVim 初始化成功，请运行 :LazyHealth 检查配置");
    Ok(())
}

pub fn enable_all_sources() -> Result<()> {
    if PlatformType::current() != PlatformType::Macos {
        println!("当前不是 macOS，跳过开启所有来源");
        return Ok(());
    }
    run_interactive_shell("sudo spctl --master-disable")
}

fn write_marker(settings: &Settings, name: &str) -> Result<()> {
    std::fs::create_dir_all(settings.status_dir())
        .map_err(|err| io_error(settings.status_dir(), err))?;
    let marker = settings.status_dir().join(format!("{name}.status"));
    std::fs::write(&marker, "completed\n").map_err(|err| io_error(marker, err))
}

#[derive(Clone, Debug)]
struct MacOptimization {
    command: &'static str,
    description: &'static str,
}

fn macos_optimizations() -> Vec<MacOptimization> {
    vec![
        MacOptimization {
            command: "pwpolicy -clearaccountpolicies",
            description: "设置密码策略",
        },
        MacOptimization {
            command: "sudo nvram SystemAudioVolume=\" \"",
            description: "关闭开机声音",
        },
        MacOptimization {
            command: "defaults write com.apple.menuextra.battery ShowPercent -bool true",
            description: "电池显示百分比",
        },
        MacOptimization {
            command: "defaults write NSGlobalDomain KeyRepeat -int 3",
            description: "设置键盘按键重复延迟",
        },
        MacOptimization {
            command: "defaults write NSGlobalDomain NSAutomaticSpellingCorrectionEnabled -bool false",
            description: "禁止自动拼写纠正",
        },
        MacOptimization {
            command: "defaults write com.apple.finder ShowStatusBar -bool true",
            description: "Finder 显示状态栏",
        },
        MacOptimization {
            command: "defaults write com.apple.finder ShowPathbar -bool true",
            description: "Finder 显示地址栏",
        },
        MacOptimization {
            command: "defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true",
            description: "禁止在网络驱动器上生成 .DS_Store 文件",
        },
        MacOptimization {
            command: "sudo spctl --master-disable",
            description: "开启所有来源",
        },
        MacOptimization {
            command: "defaults write com.apple.finder _FXShowPosixPathInTitle -bool YES",
            description: "减弱 Finder 动态效果",
        },
        MacOptimization {
            command: "defaults write com.apple.dock workspaces-swoosh-animation-off -bool YES",
            description: "减弱 Dock 动态效果",
        },
    ]
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MacOptimizationStatus {
    #[serde(default)]
    executed_hashes: BTreeSet<String>,
}

fn load_macos_status(settings: &Settings) -> Result<MacOptimizationStatus> {
    let path = settings.mac_optimization_status_file();
    if !path.exists() {
        return Ok(MacOptimizationStatus::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|err| io_error(&path, err))?;
    Ok(serde_json::from_str(&content)?)
}

fn save_macos_status(settings: &Settings, status: &MacOptimizationStatus) -> Result<()> {
    let path = settings.mac_optimization_status_file();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    }
    let content = serde_json::to_string_pretty(status)?;
    std::fs::write(&path, format!("{content}\n")).map_err(|err| io_error(path, err))
}

fn hash_command(command: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(command.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[allow(dead_code)]
fn shell_join_command(program: &str, args: &[&str]) -> String {
    std::iter::once(program)
        .chain(args.iter().copied())
        .map(quote_sh)
        .collect::<Vec<_>>()
        .join(" ")
}
