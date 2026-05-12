# az-plugin-runtime

AddZero 插件系统的运行时管理库，负责插件包的发现、校验、安装、实例化及市场快照生成。

## 功能

- **目录扫描** — 自动扫描指定目录中的 `.azplugin` 格式插件包并解析清单
- **包校验** — 安装前通过 `checksums.sha256` 对包内文件进行 SHA256 完整性校验
- **安全解压** — 解压插件包到目标目录，内置路径逃逸（Zip Slip）防护
- **插件安装** — 从目录安装插件，按 `插件ID/版本号` 组织安装目录
- **实例管理** — 为已安装插件创建运行时实例，基于 slug 自动生成唯一标识并检测冲突
- **市场快照** — 聚合目录中所有插件的安装状态、标签和实例计数，生成 `MarketplaceSnapshot`
- **开发态打包** — `ensure_dev_package` 从源目录快速生成 `.azplugin` 压缩包，方便开发调试
- **错误处理** — 通过 `thiserror` 定义完整的 `RuntimeError` 枚举，覆盖包缺失、重复安装、校验失败等场景

## 安装

在项目的 `Cargo.toml` 中添加依赖：

```toml
# 本地路径引用（monorepo 开发态）
[dependencies]
az-plugin-runtime = { path = "crates/runtime/az-plugin-runtime" }

# crates.io 引用（发布态）
[dependencies]
az-plugin-runtime = "0.1"
```

## 用法

```rust
use az_plugin_runtime::PluginRuntime;
use std::path::Path;

// 1. 创建运行时，指定插件包目录和安装根目录
let mut runtime = PluginRuntime::new(
    Path::new("/path/to/catalog"),
    Path::new("/path/to/installed"),
)?;

// 2. 从目录安装指定插件
let descriptor = runtime.install_from_catalog("my-plugin-id")?;

// 3. 为已安装插件创建运行时实例
let instance = runtime.create_instance("my-plugin-id", "我的插件实例")?;

// 4. 获取市场快照（可用/已安装标签、实例计数等）
let snapshot = runtime.marketplace_snapshot();
for entry in &snapshot.entries {
    println!("{}: {:?}", entry.name, entry.status);
}

// 5. 开发态快速打包
runtime.ensure_dev_package(
    Path::new("/path/to/plugin/source"),
    "my-plugin",
)?;
```

## 依赖的 crates

| crate | 用途 |
|-------|------|
| `az-plugin-contract` | 插件系统核心契约类型（`PluginDescriptor`、`PluginInstance`、`MarketplaceSnapshot` 等） |
| `chrono` | UTC 时间戳（`DateTime<Utc>`），用于实例创建时间 |
| `serde` | 序列化/反序列化框架 |
| `sha2` | SHA256 哈希计算，用于包文件完整性校验 |
| `thiserror` | 派生 `RuntimeError` 错误枚举 |
| `toml_edit` | 解析插件包内的 `plugin.toml` 清单文件 |
| `uuid` | 生成唯一 slug 后缀，避免实例标识冲突 |
| `zip` | 读写 `.azplugin` 格式的 ZIP 压缩包 |
