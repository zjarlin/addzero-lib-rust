# az-starter-organization

组织中心系统插件，维护部门结构、团队归属与责任人信息。

## 功能

- 以表格方式展示部门名称、上级部门、负责人与成员数
- 注册到「系统插件」菜单分区，提供「部门管理」入口页
- 组织树与责任域作为 RBAC 权限模型的基础数据维度
- 兼容 web 与 desktop 两种运行环境

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-organization = { path = "../az-starter-organization" }       # workspace 内部引用
# 或发布后：
# az-starter-organization = "0.1"                                        # crates.io 引用
```

## 用法

本 crate 通常不直接调用，而是通过宿主应用统一链接：

```rust
// 宿主应用在启动时调用 link_all()，自动注册所有系统插件
az_system_starters::link_all();

// 插件注册中心会在运行时发现 OrganizationStarter，
// 并在「系统插件」分区下创建「部门管理」菜单项与对应的表格页面
```

如需单独确保本 crate 被链接器保留：

```rust
az_starter_organization::ensure_linked();
```

## 依赖的 crates

- `az-plugin-contract` — 插件描述符、页面 schema、菜单贡献等核心协议定义
- `az-plugin-macros` — `#[az_starter]` 过程宏，自动生成插件注册入口
- `az-plugin-registry` — 插件注册中心，提供 `PluginStarter` trait 与全局注册表
