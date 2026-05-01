use clap::Args;

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// 软件包名，多个包可以传多个参数或用逗号分隔
    #[arg(required = true, value_delimiter = ',')]
    pub packages: Vec<String>,
    /// 只更新本地，不自动 git push
    #[arg(long)]
    pub no_push: bool,
}
