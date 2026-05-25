# az-desktop-plugin-registry

基于 `inventory` crate 的编译期插件注册机制，自动发现并加载所有实现了 `DesktopPlugin` trait 的插件。

## 功能

- 使用 `inventory::collect!` 在编译期收集所有 `DesktopPluginRegistration` 提交
- `load_plugins()` — 加载所有已注册插件，按名称排序返回
- 零手动注册：业务插件只需调用 `register_desktop_plugin!` 即可被自动发现

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-desktop-plugin-registry = { path = "../az-desktop-plugin-registry" }       # workspace 内部引用
# 或发布后：
# az-desktop-plugin-registry = "0.1"                                            # crates.io 引用
```

## 用法

**注册插件：**

```rust
use az_desktop_plugin::{
    DesktopInitContext, DesktopEvent, DesktopExecContext,
    DesktopViewContext, DesktopRenderLayer, Plugin, EventPropagation,
};
use az_desktop_plugin_registry::register_desktop_plugin;

#[derive(Default)]
struct MyPlugin;

impl Plugin<DesktopInitContext, DesktopEvent, DesktopExecContext, DesktopViewContext, DesktopRenderLayer>
    for MyPlugin
{
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn render_layer(&self) -> DesktopRenderLayer {
        DesktopRenderLayer::Main
    }
}

register_desktop_plugin!(MyPlugin);
```

**加载所有插件：**

```rust
use az_desktop_plugin_registry::load_plugins;

let plugins = load_plugins();
for plugin in &plugins {
    println!("Loaded: {}", plugin.name());
}
```

## 依赖的 crates

- `az-desktop-plugin` — 插件 trait 定义和上下文类型
- `inventory` — 编译期分布式切片收集机制
