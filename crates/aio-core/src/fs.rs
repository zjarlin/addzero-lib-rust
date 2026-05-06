//! 跨平台文件系统抽象。
//!
//! 提供统一的文件读写、目录遍历、临时文件管理接口，
//! 适配权限沙箱下的受限文件访问。

/// Placeholder — will wrap tokio::fs with sandbox-aware access control.
pub use tokio::fs;
