# az-starter-identity

用户中心系统插件，负责用户、角色与开发环境默认登录管理。

## 功能

- 提供用户管理的表格化界面（用户名、角色、状态、登录源）
- 内置开发环境默认用户 `admin`（管理员）和 `luna`（审核员）
- 开发环境默认 `admin/admin` 本地登录，生产环境支持环境变量配置
- 作为系统插件自动挂载到「系统插件 → 用户管理」菜单入口

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-identity = { path = "../az-starter-identity" }  # workspace 内部引用
# 或发布后：
# az-starter-identity = "0.1"                               # crates.io 引用
```

## 用法

```rust
use az_plugin_registry::PluginStarter;
use az_starter_identity::ensure_linked;

// 确保链接器不会优化掉本 crate 的注册函数
ensure_linked();

// 插件通过 #[az_starter] 宏自动注册，宿主壳子通过 PluginRegistry 统一发现
```

## 依赖的 crates

- `az-plugin-contract` — 插件描述符、页面 schema、菜单贡献等核心协议类型
- `az-plugin-macros` — 提供 `#[az_starter]` 注册宏
- `az-plugin-registry` — 插件注册中心，定义 `PluginStarter` trait
