# az-admin-plugin-registry

基于 `inventory` 的编译期 Admin 插件注册表，为 admin 工作面提供双轴上下文导航树的声明式注册与查询。

## 功能

- **编译期注册** — 通过 `register_admin_domain!`、`register_admin_branch!`、`register_admin_page!` 宏零成本声明导航结构
- **Starter 插件入口** — 通过 `declare_admin_root_page_plugin!` 组合根页面注册和链接保活入口
- **双轴上下文树** — 主轴（域）承载业务域切换，侧轴（导航节点）承载模块/页面菜单
- **动态路径匹配** — 支持 `:param` 风格的动态路径段，如 `/users/:id`
- **权限声明** — 每个节点通过 `permissions_any_of` 字段声明所需权限
- **稳定排序** — 导航节点按 `order → label → id` 排序，保证渲染顺序一致
- **空域自动清理** — 没有注册任何导航节点的域会被自动过滤

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-admin-plugin-registry = { path = "../admin-plugin-registry" }       # workspace 内部引用
# 或发布后：
# az-admin-plugin-registry = "0.1"                                         # crates.io 引用
```

## 用法

```rust
use az_admin_plugin_registry::{
    register_admin_domain, register_admin_page,
};
use az_admin_plugin_registry::navigation::{registered_domains, section_for_path};

// 声明一个业务域
register_admin_domain! {
    id: "system",
    label: "系统管理",
    order: 30,
    default_href: "/system/users",
}

// 声明一个页面节点
register_admin_page! {
    id: "system-users",
    domain: "system",
    parent: None,
    label: "用户管理",
    order: 10,
    href: "/system/users",
    active_patterns: &["/system/users", "/system/users/:id"],
    permissions_any_of: &["system:user"],
}

// 运行时查询
let domains = registered_domains();
let section = section_for_path("/system/users/42");
```

只暴露一个根级页面的 starter crate 可以使用组合宏：

```rust
az_admin_plugin_registry::declare_admin_root_page_plugin! {
    id: "system-users",
    domain: "system",
    label: "用户管理",
    order: 10,
    href: "/system/users",
}
```

## 依赖的 crates

- `inventory` — 编译期静态注册收集
