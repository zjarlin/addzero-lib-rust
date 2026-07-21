# az-desktop-plugin-registry

基于 `inventory` crate 的编译期插件注册机制，自动发现并加载所有实现了 `DesktopPlugin` trait 的插件。

## 功能

- 使用 Rudi 自动 provider 注册表在编译期收集所有桌面插件
- `load_plugins(&mut Context)` — 从 Rudi 上下文加载所有已注册插件，按名称排序返回
- 零手动注册：业务插件只需调用 `register_desktop_plugin!` 即可被自动发现

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-desktop-plugin-registry = { path = "../desktop-plugin-registry" }       # workspace 内部引用
# 或发布后：
# az-desktop-plugin-registry = "0.1"                                            # crates.io 引用
```

## 用法

**注册插件：**

```rust
use az_desktop_plugin::contract::{
    DesktopInitContext, DesktopEvent, DesktopExecContext,
    DesktopViewContext, DesktopRenderLayer, Plugin, EventPropagation,
};
use az_desktop_plugin_registry::register_desktop_plugin;
use rudi::Context;

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
use rudi::Context;

let mut context = Context::auto_register();
let plugins = load_plugins(&mut context);
for plugin in &plugins {
    println!("Loaded: {}", plugin.name());
}
```

## 依赖的 crates

- `az-desktop-plugin` — 插件 trait 定义和上下文类型
- `inventory` — 编译期分布式切片收集机制
