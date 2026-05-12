# az-starter-dictionary

字典中心系统插件，统一维护系统枚举和值域（如 `note_type`）。

## 功能

- 提供字典项的表格化管理界面（字典编码、值、显示名、用途）
- 内置 `note_type` 字典，区分智能体工作台（`flash`）与 Skill（`skill`）
- 作为系统插件自动挂载到「系统插件 → 字典管理」菜单入口

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-dictionary = { path = "../az-starter-dictionary" }  # workspace 内部引用
# 或发布后：
# az-starter-dictionary = "0.1"                                # crates.io 引用
```

## 用法

```rust
use az_plugin_registry::PluginStarter;
use az_starter_dictionary::ensure_linked;

// 确保链接器不会优化掉本 crate 的注册函数
ensure_linked();

// 插件通过 #[az_starter] 宏自动注册，宿主壳子通过 PluginRegistry 统一发现
```

## 依赖的 crates

- `az-plugin-contract` — 插件描述符、页面 schema、菜单贡献等核心协议类型
- `az-plugin-macros` — 提供 `#[az_starter]` 注册宏
- `az-plugin-registry` — 插件注册中心，定义 `PluginStarter` trait
