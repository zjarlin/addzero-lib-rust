use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct OneClickCommand {
    /// 对确认项默认回答 yes；AI 只有在用户明确授权后才应使用
    #[arg(short, long)]
    pub yes: bool,
    /// 只打印将要执行的动作，不真正执行
    #[arg(long)]
    pub dry_run: bool,
    /// 忽略已完成初始化标记，重新执行已支持状态标记的任务
    #[arg(long)]
    pub force: bool,
    #[command(subcommand)]
    pub task: OneClickTask,
}

#[derive(Clone, Copy, Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum OneClickTask {
    /// 列出所有 AI 可调用的一键任务
    List,
    /// 按旧 GraalVM CLI 的初始化任务集合逐项执行
    All,
    /// 安装/写入旧环境脚本能力
    EnvScripts,
    /// Git 初始化
    Git,
    /// Node.js 初始化
    Node,
    /// JDK 17 初始化
    Jdk,
    /// pnpm 初始化
    Pnpm,
    /// 初始化当前平台包管理器
    Pkg,
    /// 安装配置中的默认软件包
    PkgManager,
    /// GraalVM 初始化
    #[command(alias = "graavlvm")]
    Graalvm,
    /// FinalShell 初始化
    #[command(alias = "finalshell")]
    FinalShell,
    /// IntelliJ IDEA XDG 配置初始化
    Idea,
    /// Zulu JDK 初始化
    ZuluJdk,
    /// Windows PowerShell 系统优化
    Powershell,
    /// Windows PowerShell profile 和执行策略初始化
    PowershellEnv,
    /// 下载夸克网盘客户端安装包
    Quark,
    /// Docker 初始化
    Docker,
    /// LazyVim 初始化
    Lazyvim,
    /// Homebrew 初始化
    Homebrew,
    /// Oh My Zsh 初始化
    Ohmyzsh,
    /// macOS 系统优化
    Macos,
    /// macOS 开启所有来源
    EnableAllSources,
    /// keji 面板
    Keji,
}
