# az-plugin-contract

插件系统的核心契约层，定义插件宿主与插件之间共享的所有可序列化数据模型。

## 功能

- 插件描述符（`PluginDescriptor`）：声明插件的元信息、能力、页面和菜单贡献
- UI 页面模式（`PageSchema`）：支持表格、表单、详情、看板、Markdown、图谱等页面形态
- 导航与 Shell 快照（`ShellSnapshot`）：为前端 Shell 提供当前用户、导航树和统计信息
- 插件市场（`MarketplaceSnapshot`）：展示可用插件及其安装状态
- 插件实例（`PluginInstance`）：已安装插件的运行时实例及其配置
- 运行时概览（`RuntimeOverview`）：内核的全局状态摘要
- 所有类型均派生 `Clone`、`Debug`、`Serialize`、`Deserialize`，可直接用于 JSON 传输

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-plugin-contract = { path = "../az-plugin-contract" }       # workspace 内部引用
# 或发布后：
# az-plugin-contract = "0.1"                                   # crates.io 引用
```

## 用法

```rust
use az_plugin_contract::{PluginDescriptor, ShellSnapshot, PageSchema};

// 构造插件描述符
let descriptor = PluginDescriptor {
    id: "my-plugin".to_string(),
    name: "My Plugin".to_string(),
    version: "0.1.0".to_string(),
    ..Default::default()
};

// 使用页面模式
let schema = PageSchema::default(); // Markdown 空页面
```

## 依赖的 crates

- `chrono` - 日期时间处理（带 `serde` 特性）
- `serde` - 序列化与反序列化
