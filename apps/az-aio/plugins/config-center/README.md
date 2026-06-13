# az-aio-plugin-config-center

桌面端配置中心插件，集中管理 Dotfiles 监控、设备配对身份、XDG 路径以及 AI 模型提供商配置（OpenAI / Anthropic / Gemini）。

## 功能

- **Dotfiles 监控**：实时扫描 dotfiles 根目录，展示被监视文件数量、变更文件、冲突文件及关联设备列表
- **设备配对**：生成并持久化本地设备指纹与配对元数据，用于跨设备同步场景的身份标识
- **XDG 路径解析**：列出当前系统的 `data_dir`、`config_dir`、`state_dir`、`cache_dir` 等 XDG 基础目录
- **提供商管理**：展示已配置的 AI 模型提供商（OpenAI / Anthropic / Gemini）的启用状态、API Key 配置情况与默认模型
- **环境变量导入**：一键从 `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GEMINI_API_KEY` 等环境变量导入提供商密钥
- **连通性测试**：支持对单个提供商发起连通性测试，验证 API Key 是否有效
- **桌面插件注册**：通过 `az-desktop-plugin` + `az-desktop-plugin-registry` 自动注册为桌面端 Environment 域的 Machine 分支页面

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-aio-plugin-config-center = { path = "../az-aio-plugin-config-center" }       # workspace 内部引用
# 或发布后：
# az-aio-plugin-config-center = "0.1"                                            # crates.io 引用
```

## 用法

```rust,ignore
// 该 crate 作为桌面插件自动发现并注册，无需手动调用。
// 在桌面 shell 中，Config Center 页面位于 Environment > Machine 下，
// 通过 "/config" 路由访问。
//
// 底层模块可单独使用：
// use az_aio_plugin_config_center::dotfiles_monitor::scan_dotfiles_status;
// use az_aio_plugin_config_center::pairing::local_pairing_info;
// use az_aio_plugin_config_center::paths::resolve_config_center_paths;
//
// let dotfiles = scan_dotfiles_status()?;
// println!("watched: {}, changed: {}", dotfiles.watched_files, dotfiles.changed_files);
```

## 依赖的 crates

- `az-ai-agent` — 获取各提供商的默认模型名称
- `az-assets` — AI 模型提供商配置的加密存储与读写
- `az-desktop-plugin` — 桌面端插件 trait 与上下文类型
- `az-desktop-plugin-registry` — 插件注册清单，通过 inventory 自动发现
- `gpui` — Zed 生态 UI 框架，用于渲染配置面板

## 模块

- `dotfiles_monitor` — 扫描 dotfiles 目录状态
- `dotfiles_monitor_diff` — dotfiles 差异对比
- `dotfiles_monitor_types` — dotfiles 相关数据类型
- `pairing` — 设备配对标识（指纹 + 元数据）
- `paths` — XDG 基本目录路径解析