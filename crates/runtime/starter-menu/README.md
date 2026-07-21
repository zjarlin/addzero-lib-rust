# az-starter-menu

菜单中心系统导航注册 crate。

## 功能

- 注册「系统插件 → 菜单挂载」admin 导航入口
- 保留 `ensure_linked()` 作为显式链接锚点
- 页面数据由 admin 应用侧 provider 按路由加载

## 用法

```rust
az_starter_menu::navigation::ensure_linked();
```

通常由 `az_system_starters::linking::link_all()` 统一调用。

## 依赖的 crates

- `az-admin-plugin-registry` — admin 双轴上下文导航注册表
