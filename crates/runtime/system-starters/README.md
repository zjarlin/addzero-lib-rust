# az-system-starters

系统 admin 导航统一链接入口，聚合所有 addzero 系统级 starter crate。

## 功能

- 提供 `link_all()` 函数，一次性链接所有系统级 starter crate
- 注册「系统插件」admin 主轴域
- 确保链接器不会因未直接调用而剥离各 starter 的导航注册入口
- 宿主应用只需依赖本 crate，无需逐个引入每个系统 starter

## 用法

```rust
az_system_starters::linking::link_all();
// 此后 az-admin-plugin-registry 可发现系统级导航节点。
```

## 包含的 starter

| crate | 职责 |
|---|---|
| `az-starter-identity` | 用户、角色与登录管理入口 |
| `az-starter-organization` | 部门结构、团队归属与责任人入口 |
| `az-starter-dictionary` | 数据字典与枚举常量管理入口 |
| `az-starter-menu` | 系统菜单配置与路由管理入口 |
| `az-starter-audit` | 操作日志与审计追踪入口 |
| `az-starter-storage` | 上传下载与插件包仓库入口 |
