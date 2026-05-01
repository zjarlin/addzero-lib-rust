use clap::Subcommand;

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
