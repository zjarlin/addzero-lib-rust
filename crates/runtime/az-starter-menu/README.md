# az-starter-menu

菜单中心系统插件，统一输出宿主菜单结构与插件挂载说明。

## 功能

- 以 Markdown 页面展示菜单挂载机制：固定页 → 系统插件 → 业务实例
- 说明插件挂载架构约束：新增插件只补描述与注册，不再修改主路由表
- 作为系统插件自动挂载到「系统插件 → 菜单挂载」菜单入口

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-menu = { path = "../az-starter-menu" }  # workspace 内部引用
# 或发布后：
# az-starter-menu = "0.1"                           # crates.io 引用
```

## 用法

```rust
use az_plugin_registry::PluginStarter;
use az_starter_menu::ensure_linked;

// 确保链接器不会优化掉本 crate 的注册函数
ensure_linked();

// 插件通过 #[az_starter] 宏自动注册，宿主壳子通过 PluginRegistry 统一发现
```

## 依赖的 crates

- `az-plugin-contract` — 插件描述符、页面 schema、菜单贡献等核心协议类型
- `az-plugin-macros` — 提供 `#[az_starter]` 注册宏
- `az-plugin-registry` — 插件注册中心，定义 `PluginStarter` trait
