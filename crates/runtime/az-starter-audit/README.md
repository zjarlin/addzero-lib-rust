# az-starter-audit

审计中心系统插件启动器，记录插件安装、实例创建与权限变更等审计事件。

## 功能

- 以系统插件身份注册到宿主应用商店
- 提供审计日志看板页面（今日事件统计、高风险告警、最近日志列表）
- 统一记账系统插件与业务插件的关键操作

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-starter-audit = { path = "../az-starter-audit" }         # workspace 内部引用
# 或发布后：
# az-starter-audit = "0.1"                                   # crates.io 引用
```

## 用法

```rust
// 通过 ensure_linked() 确保链接器不会优化掉启动器符号
az_starter_audit::ensure_linked();

// 插件注册由 az_starter 宏自动完成，无需手动调用
```

## 依赖的 crates

- `az-plugin-contract` — 插件契约模型（描述符、页面 schema、菜单贡献等）
- `az-plugin-macros` — `#[az_starter]` 过程宏，自动生成插件注册代码
- `az-plugin-registry` — 插件注册表，提供 `PluginStarter` trait
