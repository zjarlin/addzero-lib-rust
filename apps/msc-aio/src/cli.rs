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
    /// 启动 Axum API 服务，供 Next.js / Tauri 前端调用
    ServeApi,
    /// 打印当前前后端分层迁移状态
    Status,
}
