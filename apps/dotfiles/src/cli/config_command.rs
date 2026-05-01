use clap::Subcommand;

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
