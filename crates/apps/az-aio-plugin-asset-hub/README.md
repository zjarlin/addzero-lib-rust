# az-aio-plugin-asset-hub

桌面端资产管理插件，提供资产列表浏览、Skill 扫描入库、Compose 资产种子及按 subtype 过滤的元数据面板。

## 功能

- **资产面板**：列出系统中所有资产的概要信息，按 `AssetKind`（Capture、Note、Skill、Software、Package）统计数量
- **Skill 扫描合并**：扫描 `~/.agents/skills` 目录，将 SKILL.md 文件自动导入 `az_assets` 资产库，自动识别标签（如 rust、docker、gradle 等）
- **Compose 种子**：一键插入示例 Docker Compose 资产，附带规范的 subtype 元数据
- **桌面插件注册**：通过 `az-desktop-plugin` + `az-desktop-plugin-registry` 自动注册为桌面端 Knowledge 域的插件页面
- **工具条操作**：刷新、扫描 Skill、种子 Compose 等操作通过桌面端 toolBar 按钮触发

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-plugin-asset-hub = { path = "../az-aio-plugin-asset-hub" }       # workspace 内部引用
# 或发布后：
# az-aio-plugin-asset-hub = "0.1"                                        # crates.io 引用
```

## 用法

```rust,ignore
// 该 crate 作为桌面插件自动发现并注册，无需手动调用。
// 在桌面 shell 中，Asset Hub 页面位于 Knowledge 域下，
// 通过 "/assets" 路由访问。
//
// Skill 扫描示例（由插件 toolbar 触发，这里展示底层调用链路）：
// use az_aio_plugin_asset_hub::skill_scanner::scan_skill_assets;
//
// let assets = scan_skill_assets()?;
// for asset in assets {
//     println!("{} → tags: {:?}", asset.name, asset.tags);
// }
```

## 依赖的 crates

- `az-assets` — 底层资产存储与服务抽象
- `az-desktop-plugin` — 桌面端插件 trait 与上下文类型
- `az-desktop-plugin-registry` — 插件注册清单，通过 inventory 自动发现
- `gpui` — Zed 生态 UI 框架，用于渲染资产面板

## 模块

- `skill_scanner` — 扫描 `~/.agents/skills/` 目录，解析 YAML frontmatter 并提取标签