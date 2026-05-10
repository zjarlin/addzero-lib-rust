//! 文件系统 I/O 工具库，提供「移动并符号链接回原位」以及路径确保操作。
//!
//! # 核心功能
//!
//! - [`mvln`] — 将文件/目录移动到新位置，并在原路径创建符号链接指向新位置。
//!   适用于将本地数据迁移到外部存储后保留原有访问路径的场景。
//! - [`undo_mvln`] — 撤销 `mvln` 操作：移除符号链接并将文件移回原位。
//! - [`PathExt`] trait — 为 [`std::path::Path`] 扩展三个常用方法：
//!   - [`ensure_file`](PathExt::ensure_file) — 确保路径存在且为文件（不存在则自动创建）。
//!   - [`ensure_dir`](PathExt::ensure_dir) — 确保路径存在且为目录（不存在则自动创建）。
//!   - [`remove_if_exists`](PathExt::remove_if_exists) — 安全删除路径（不存在时静默通过）。
//! - [`MoveLink`] — builder 风格的 `mvln` 包装器，适合链式调用。
//!
//! # 错误处理
//!
//! 所有公开函数返回 [`Result<T, IoError>`]，[`IoError`] 使用 `thiserror` 派生，
//! 提供结构化的错误变体：路径缺失、目标缺失、文件类型不符、符号链接相关错误等。
//!
//! # 平台说明
//!
//! 符号链接操作仅在 Unix 平台可用；非 Unix 平台调用时返回
//! [`IoError::UnsupportedSymlink`]。
//!
//! # 典型用法
//!
//! ```rust,no_run
//! use az_io::mvln;
//! use std::path::Path;
//!
//! // 将 data.db 移动到 /mnt/external/data.db，并在原位保留符号链接
//! let new_path = mvln("data.db", "/mnt/external")?;
//! ```
