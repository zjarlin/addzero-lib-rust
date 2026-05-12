# az-starter-storage

文件中心系统插件，管理上传下载能力与插件包仓库。

## 功能

- 以表格方式展示包仓库的存储架构：PostgreSQL 存元数据，本地文件/Git 存工件
- 支持 `.wasm` 业务插件上传，页面、菜单、维护者等元数据写入 PostgreSQL
- 运行时二进制、脚本文件对象与发布归档走本地文件或 Git 发布
- 注册到「系统插件」菜单分区，提供「包仓库」管理入口
- 兼容 web 与 desktop 两种运行环境

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-storage = { path = "../az-starter-storage" }       # workspace 内部引用
# 或发布后：
# az-starter-storage = "0.1"                                   # crates.io 引用
```

## 用法

本 crate 通常不直接调用，而是通过宿主应用统一链接：

```rust
// 宿主应用在启动时调用 link_all()，自动注册所有系统插件
az_system_starters::link_all();

// 插件注册中心会在运行时发现 StorageStarter，
// 并在「系统插件」分区下创建「包仓库」菜单项与对应的表格页面
```

如需单独确保本 crate 被链接器保留：

```rust
az_starter_storage::ensure_linked();
```

## 依赖的 crates

- `az-plugin-contract` — 插件描述符、页面 schema、菜单贡献等核心协议定义
- `az-plugin-macros` — `#[az_starter]` 过程宏，自动生成插件注册入口
- `az-plugin-registry` — 插件注册中心，提供 `PluginStarter` trait 与全局注册表
