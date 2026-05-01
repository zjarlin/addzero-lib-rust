use clap::Subcommand;

use super::{
    AddDotfilesCommand, ConfigCommand, InitCommand, OneClickCommand, PackageCommand,
    PackageSubcommand, RemoveDotfilesCommand,
};

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
    /// 打印配置 JSON，兼容旧 GraalVM CLI 的 cat-config
    CatConfig,
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
    #[command(alias = "rm-dotfiles")]
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
    /// 一键调用旧 dotfiles-cli-graalvm 初始化任务，由 AI 负责编排
    #[command(name = "oneclick", alias = "task", alias = "auto-init")]
    Oneclick(OneClickCommand),
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
