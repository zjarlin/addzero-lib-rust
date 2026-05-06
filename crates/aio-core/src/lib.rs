//! AIO 内核层 — 跨平台系统抽象
//!
//! 只包含：事件总线、文件 IO、进程调度、网络抽象、权限沙箱。
//! 不包含任何业务逻辑、脚本逻辑、AI 逻辑。

pub mod event;
pub mod fs;
pub mod net;
pub mod process;
pub mod sandbox;

/// Core-wide error type.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
