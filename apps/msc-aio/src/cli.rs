use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "msc-aio")]
#[command(about = "MSC_AIO native desktop and CLI entrypoint", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 启动桌面端（默认行为）
    Desktop,
    /// 打印当前 native 架构迁移状态
    Status,
}
