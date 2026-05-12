# az-plugin-kernel

插件系统的运行时内核，负责插件生命周期管理、服务注入和前端 Shell 数据聚合。

## 功能

- `PlatformKernel`：统一管理插件的安装、实例创建、目录刷新和页面解析
- 基于 `shaku` 的依赖注入：`AuthProvider`、`RbacService`、`DictionaryService`、`AuditService`、`StorageService` 五大服务接口
- 开发模式默认实现：`DevAuthProvider`（admin/admin）、`AllowAllRbacService`（全权限放行）
- `ShellSnapshot` 聚合：从注册表和运行时中组合当前用户的导航树、插件计数和认证模式
- `MarketplaceSnapshot` 聚合：合并系统插件与已安装业务插件，按名称排序
- 页面解析：支持系统页面和实例页面两种作用域
- 错误类型 `KernelError`：统一包装运行时错误和锁中毒

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-plugin-kernel = { path = "../az-plugin-kernel" }       # workspace 内部引用
# 或发布后：
# az-plugin-kernel = "0.1"                                 # crates.io 引用
```

## 用法

```rust
use az_plugin_kernel::PlatformKernel;

// 创建内核实例
let kernel = PlatformKernel::new("catalog", "plugins/host")?;

// 获取 Shell 快照
let snapshot = kernel.shell_snapshot()?;
println!("当前用户: {}", snapshot.actor.display_name);

// 安装插件
let descriptor = kernel.install_catalog_plugin("my-plugin")?;

// 创建插件实例
let instance = kernel.create_instance("my-plugin", "生产环境")?;
```

## 依赖的 crates

- `az-plugin-contract` - 插件契约数据模型
- `az-plugin-registry` - 插件注册表
- `az-plugin-runtime` - 插件运行时
- `shaku` - 依赖注入框架
- `thiserror` - 错误类型派生
