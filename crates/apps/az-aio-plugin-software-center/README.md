# az-aio-plugin-software-center

Software Center 插件 — 扫描本地安装包、与软件目录关联、提供归档整理能力，集成在 AIO Desktop 的知识工作区中。

## 功能

- **安装包扫描**：扫描 Downloads 和 Desktop 目录，识别 `.dmg`、`.pkg`、`.app` 等安装包文件
- **目录关联**：扫描结果与 `az-software-catalog` 中的软件条目自动匹配（按 slug / 标题模糊匹配）
- **归档整理**：将识别到的安装包移动到 software-center 存储目录，按平台和架构归档
- **AIO Desktop 集成**：注册为桌面插件，位于 Knowledge → Software 导航下，提供 Refresh / Scan / Organize 工具栏操作

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-plugin-software-center = { path = "../az-aio-plugin-software-center" }  # workspace 内部引用
# 或发布后：
# az-aio-plugin-software-center = "2026.5.10"                                   # crates.io 引用
```

## 用法

本 crate 是桌面插件，由 AIO Desktop 通过 plugin registry 自动发现和加载：

```rust,no_run
use az_aio_plugin_software_center::installer_scanner::{installer_matches_catalog, InstallerPackage};

let package = InstallerPackage {
    id: "1".to_string(),
    file_name: "raycast-1.2.3-macos-arm64.dmg".to_string(),
    source_path: String::new(),
    version: "1.2.3".to_string(),
    platform: "macOS".to_string(),
    arch: "arm64".to_string(),
    target_path: String::new(),
    install_status: "unconfirmed".to_string(),
    status: "pending".to_string(),
    md5: "x".to_string(),
};

// 检查安装包是否匹配软件目录中的条目
assert!(installer_matches_catalog(&package, "raycast", "Raycast"));
```

## 依赖的 crates

- `az-desktop-plugin` — 桌面插件框架 trait 定义
- `az-desktop-plugin-registry` — 插件自动注册（`inventory` 机制）
- `az-software-catalog` — 软件目录数据模型，提供 `SoftwareCatalogDto`
- `gpui` — GPU 加速 GUI 框架，渲染页面视图
- `md5` — 安装包文件哈希计算
- `serde` — 序列化支持