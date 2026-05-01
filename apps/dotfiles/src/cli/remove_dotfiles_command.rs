use clap::Args;

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
