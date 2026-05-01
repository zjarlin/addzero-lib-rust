use clap::Args;

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
