# az-system-starters

系统插件统一链接入口，聚合所有 addzero 系统级 starter 插件。

## 功能

- 提供 `link_all()` 函数，一次性链接所有系统级 starter 插件
- 确保链接器不会因未直接调用而剥离各 starter 的注册入口
- 宿主应用只需依赖本 crate，无需逐个引入每个系统插件

### 包含的系统插件

| 插件 | 职责 |
|------|------|
| `az-starter-identity` | 用户、角色与登录管理 |
| `az-starter-organization` | 部门结构、团队归属与责任人 |
| `az-starter-dictionary` | 数据字典与枚举常量管理 |
| `az-starter-menu` | 系统菜单配置与路由管理 |
| `az-starter-audit` | 操作日志与审计追踪 |
| `az-starter-storage` | 上传下载与插件包仓库 |

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-system-starters = { path = "../az-system-starters" }       # workspace 内部引用
# 或发布后：
# az-system-starters = "0.1"                                   # crates.io 引用
```

## 用法

```rust
fn main() {
    // 一次性链接所有系统级插件
    az_system_starters::link_all();

    // 此后插件注册中心可发现所有系统级插件，
    // 并在运行时组装菜单与页面
}
```

## 依赖的 crates

- `az-starter-identity` — 用户、角色与登录管理插件
- `az-starter-organization` — 部门结构、团队归属与责任人插件
- `az-starter-dictionary` — 数据字典与枚举常量管理插件
- `az-starter-menu` — 系统菜单配置与路由管理插件
- `az-starter-audit` — 操作日志与审计追踪插件
- `az-starter-storage` — 上传下载与插件包仓库插件
