use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aio")]
#[command(about = "AIO backend API server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 启动 API 后端服务
    Serve,
    /// 打印当前架构状态
    Status,
}
