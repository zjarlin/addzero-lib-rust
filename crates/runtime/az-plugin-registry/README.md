# az-plugin-registry

插件注册中心，统一管理系统插件与业务插件的描述符、实例及运行时导航。

## 功能

- **编译期自动收集**：基于 `inventory` 宏在编译期自动收集所有插件启动器，无需手动维护插件列表。
- **双轨插件管理**：系统插件（System）与业务插件（Business）分轨存储，互不干扰。
- **实例管理**：业务插件可拥有多个实例（`PluginInstance`），按 slug 唯一标识。
- **市场条目聚合**：`marketplace_entries()` 将两类插件统一转换为 `MarketplaceEntry` 列表，按名称排序。
- **导航生成**：`plugin_navigation()` 自动生成按「系统插件」和「业务应用」分组的导航结构。
- **页面解析**：`resolve_system_page()` / `resolve_instance_page()` 根据插件 ID 和页面 ID 解析出完整的 `ResolvedPage`，包含面包屑和 schema 信息。

## 安装

### 使用本地路径（开发环境）

```toml
[dependencies]
az-plugin-registry = { path = "crates/runtime/az-plugin-registry" }
```

### 使用 crates.io（发布后）

```toml
[dependencies]
az-plugin-registry = "0.1.0"
```

## 用法

### 注册一个插件启动器

```rust
use az_plugin_contract::PluginDescriptor;
use az_plugin_registry::{PluginStarter, StarterRegistration, inventory};

struct MyPluginStarter;

impl PluginStarter for MyPluginStarter {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "my-plugin".to_string(),
            name: "我的插件".to_string(),
            // ... 其他字段
        }
    }
}

// 通过 inventory 宏在编译期注册
inventory::submit! {
    StarterRegistration {
        constructor: || Box::new(MyPluginStarter),
    }
}
```

### 使用注册中心

```rust
use az_plugin_registry::{load_system_descriptors, PluginRegistry};

// 加载所有编译期注册的系统插件描述符
let system_descriptors = load_system_descriptors();

// 创建注册中心
let mut registry = PluginRegistry::new(system_descriptors);

// 注册业务插件和实例
// registry.replace_business_plugins(business_descriptors);
// registry.replace_instances(instances);

// 获取市场条目
let entries = registry.marketplace_entries();

// 生成导航结构
let navigation = registry.plugin_navigation();

// 解析系统页面
// let page = registry.resolve_system_page("plugin-id", "page-id");
```

## 依赖的 crates

| crate | 用途 |
|---|---|
| `az-plugin-contract` | 插件契约层，提供 `PluginDescriptor`、`PluginInstance`、`ResolvedPage`、`NavigationSection` 等共享类型 |
| `inventory` | 编译期插件自动收集，通过 `collect!` / `submit!` 宏实现零成本插件注册 |
