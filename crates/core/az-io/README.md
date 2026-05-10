# az-io

文件系统 I/O 工具库，提供「移动并符号链接回原位」（mvln）和路径确保等便捷操作。

## 功能

- **`mvln`** — 将文件/目录移动到新位置，并在原路径创建符号链接指向新位置，适用于数据迁移后保留原有访问路径
- **`undo_mvln`** — 撤销 mvln 操作：移除符号链接并将文件移回原位
- **`PathExt` trait** — 为 `Path` 扩展三个常用方法：
  - `ensure_file()` — 确保路径存在且为文件，不存在则自动创建
  - `ensure_dir()` — 确保路径存在且为目录，不存在则自动创建
  - `remove_if_exists()` — 安全删除路径，不存在时静默通过
- **`MoveLink`** — builder 风格的 mvln 包装器，支持链式调用
- **结构化错误** — `IoError` 使用 thiserror 派生，提供语义清晰的错误变体

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-io = { path = "../az-io" }       # workspace 内部引用
# 或发布后：
# az-io = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_io::{mvln, MoveLink, PathExt};
use std::path::Path;

// 将文件移动到新位置并符号链接回原位
let new_path = mvln("data.db", "/mnt/external")?;

// 使用 builder 风格
let new_path = MoveLink::new("data.db").to("/mnt/external").move_and_link()?;

// 撤销操作
az_io::undo_mvln("data.db")?;

// 确保目录存在
Path::new("./output/logs").ensure_dir()?;

// 确保文件存在（自动创建父目录）
Path::new("./output/config.json").ensure_file()?;

// 安全删除
Path::new("./temp").remove_if_exists()?;
```

## 依赖的 crates

- `thiserror` — 错误类型派生

## 平台说明

符号链接操作仅在 Unix 平台可用；非 Unix 平台调用 `mvln` / `MoveLink` 时返回 `UnsupportedSymlink` 错误。
