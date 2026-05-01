use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "dotfiles")]
#[command(about = "Rust port of the Addzero dotfiles CLI", long_about = None)]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum Command {
    /// 启动交互式命令行
    Repl,
    /// 管理配置文件
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// 拉取远端配置并应用软链接
    Sync {
        /// 遇到本地已有文件时备份后覆盖为软链接
        #[arg(long)]
        force_links: bool,
        /// 当本地仓库 remote 与配置不一致时，删除本地同步目录并重新克隆
        #[arg(long)]
        replace_mismatch: bool,
    },
    /// 提交并推送同步目录
    Push {
        #[arg(short, long, default_value = "Update dotfiles")]
        message: String,
    },
    /// 添加本地文件或目录到 dotfiles 仓库，并在原位置创建软链接
    AddDotfiles(AddDotfilesCommand),
    /// 移除 dotfiles 软链接配置
    RemoveDotfiles(RemoveDotfilesCommand),
    /// 添加软件包到当前平台配置
    AddPkg(PackageCommand),
    /// 从当前平台配置移除软件包
    RmPkg(PackageCommand),
    /// 调用当前平台包管理器
    Package {
        #[command(subcommand)]
        command: PackageSubcommand,
    },
    /// 查看当前平台链接和包配置
    Status,
    /// 查看初始化任务状态
    CatStatus,
    /// 查看端口占用
    ShowPort { port: u16 },
    /// 杀掉占用端口的进程
    KillPort { port: u16 },
    /// 设置 Git 全局用户名和邮箱
    GitConfig { username: String, email: String },
    /// 运行常用初始化任务
    Init(InitCommand),
    /// 通过 linuxmirrors.cn 脚本初始化 Docker
    InitDocker,
    /// 初始化 LazyVim starter 配置
    InitLazyvim,
    /// 初始化 Homebrew
    InitHomebrew,
    /// 初始化 Oh My Zsh 和常用插件
    InitOhmyzsh,
    /// macOS 开启所有来源
    EnableAllSources,
}

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum ConfigCommand {
    /// 打印配置 JSON
    Cat,
    /// 设置 Git 远端仓库地址
    SetCloudUrl { url: String },
    /// 设置同步目录
    SetSyncDir { dir: String },
    /// 设置当前平台包管理器名称
    SetPackageManager { name: String },
}

#[derive(Debug, Args)]
pub struct AddDotfilesCommand {
    /// 要添加的路径，多个路径可以传多个参数或用逗号分隔
    #[arg(required = true, value_delimiter = ',')]
    pub paths: Vec<String>,
    /// 按绝对路径解析；默认相对 HOME
    #[arg(long)]
    pub abs: bool,
    /// 只更新本地，不自动 git push
    #[arg(long)]
    pub no_push: bool,
}

#[derive(Debug, Args)]
pub struct RemoveDotfilesCommand {
    /// 要移除的源路径，多个路径可以传多个参数或用逗号分隔
    #[arg(required = true, value_delimiter = ',')]
    pub paths: Vec<String>,
    /// 按绝对路径解析；默认相对 HOME
    #[arg(long)]
    pub abs: bool,
    /// 只更新本地，不自动 git push
    #[arg(long)]
    pub no_push: bool,
}

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// 软件包名，多个包可以传多个参数或用逗号分隔
    #[arg(required = true, value_delimiter = ',')]
    pub packages: Vec<String>,
    /// 只更新本地，不自动 git push
    #[arg(long)]
    pub no_push: bool,
}

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PackageSubcommand {
    /// 安装指定软件包；不传则安装配置中的默认包
    Install { packages: Vec<String> },
    /// 搜索软件包
    Search { keyword: String },
    /// 查看已安装包版本
    Version { package: String },
}

#[derive(Debug, Args)]
pub struct InitCommand {
    /// 对确认项默认回答 yes
    #[arg(short, long)]
    pub yes: bool,
    /// 忽略已完成初始化标记，重新执行
    #[arg(long)]
    pub force: bool,
    /// 初始化时安装配置中的默认包
    #[arg(long)]
    pub install_packages: bool,
    /// 初始化时拉取远端并应用链接
    #[arg(long)]
    pub sync: bool,
    /// sync 时备份已有文件并覆盖为软链接
    #[arg(long)]
    pub force_links: bool,
    /// sync 时允许替换 remote 不一致的本地同步目录
    #[arg(long)]
    pub replace_mismatch: bool,
    /// 运行 macOS defaults 优化项
    #[arg(long)]
    pub macos_optimize: bool,
}
