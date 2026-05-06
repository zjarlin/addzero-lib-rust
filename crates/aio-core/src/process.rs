//! 跨平台进程调度抽象。
//!
//! 统一管理子进程的生命周期（超时、权限、环境变量、工作目录）。
//! 用于托管 Bash/Python 等外部脚本运行时。

/// Placeholder — will wrap tokio::process with lifecycle management.
pub use tokio::process;
