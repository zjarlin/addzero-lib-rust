# az-desktop-plugin

桌面插件系统的核心 trait 和注册基础设施，为基于 GPUI 的桌面应用提供可扩展的插件架构。

## 功能

- 定义 [`Plugin`] trait，统一插件的生命周期：初始化（`setup`）、事件处理（`on_event`）和视图渲染（`render`）
- `DesktopInitContext` — 在 setup 阶段注册 domain、branch、page、toolbar action、summary card 和 command 等 UI 贡献
- `DesktopHostRegistry` — 聚合所有插件贡献，提供路由查询、domain 导航树、分支层次结构等查询接口
- `DesktopExecContext` — 事件执行上下文，支持通知（notify）、刷新触发（request_refresh）、选中实体设置和路由导航
- `DesktopHostServices` trait — 桌面宿主服务抽象，统一暴露 drive、asset、AI provider 和 software catalog 等后端能力
- 事件系统：`DesktopEvent` 枚举覆盖启动、路由变更、action 调用、选中变更、刷新和定时 Tick 等场景

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-desktop-plugin = { path = "../az-desktop-plugin" }       # workspace 内部引用
# 或发布后：
# az-desktop-plugin = "0.1"                                  # crates.io 引用
```

## 用法

```rust
use az_desktop_plugin::api::{
    DesktopInitContext, DesktopEvent, DesktopExecContext,
    DesktopViewContext, DesktopRenderLayer, EventPropagation, Plugin,
};
use gpui::AnyElement;

struct MyPlugin;

impl Plugin<DesktopInitContext, DesktopEvent, DesktopExecContext, DesktopViewContext, DesktopRenderLayer>
    for MyPlugin
{
    fn name(&self) -> &'static str {
        "my-plugin"
    }

    fn setup(&mut self, ctx: &mut DesktopInitContext) {
        ctx.register_domain("my-domain", "My Domain", 10, "/home");
        ctx.register_page("home", "my-domain", None::<String>, "Home", "主页", "/home", 10);
    }

    fn on_event(
        &mut self,
        event: &DesktopEvent,
        ctx: &mut DesktopExecContext,
    ) -> EventPropagation {
        match event {
            DesktopEvent::ActionInvoked { action_id, .. } if action_id == "my.refresh" => {
                ctx.request_refresh();
                EventPropagation::Stop
            }
            _ => EventPropagation::Continue,
        }
    }

    fn render(&mut self, _ctx: &mut DesktopViewContext) -> Option<AnyElement> {
        None
    }

    fn render_layer(&self) -> DesktopRenderLayer {
        DesktopRenderLayer::Main
    }
}
```

## 依赖的 crates

- `az-assets` — 资产和图数据模型
- `az-drive-agent` — 同步驱动代理定义
- `az-drive-store` — 同步任务状态和冲突模型
- `az-software-catalog` — 软件目录 DTO
- `gpui` — UI 框架（`AnyElement` 等类型）
- `serde_json` / `uuid` — 通用数据承载
