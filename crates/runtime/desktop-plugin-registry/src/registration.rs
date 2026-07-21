#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use az_desktop_plugin::contract::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopPlugin, DesktopRenderLayer,
    DesktopViewContext, Plugin,
};

#[doc(hidden)]
/// 从 Rudi 上下文加载全部 desktop 插件。
///
/// 返回列表按插件名排序，保证宿主 shell 每次启动时顺序稳定。
pub fn load_plugins(context: &mut rudi::Context) -> Vec<Box<DesktopPlugin>> {
    let mut plugins = context.resolve_by_type::<Box<DesktopPlugin>>();
    plugins.sort_by(|left, right| left.name().cmp(right.name()));
    plugins
}

/// 为可默认构造的 desktop 插件创建 Rudi transient provider。
pub fn desktop_plugin_provider<P>() -> rudi::TransientProvider<Box<DesktopPlugin>>
where
    P: Default
        + Plugin<
            DesktopInitContext,
            DesktopEvent,
            DesktopExecContext,
            DesktopViewContext,
            DesktopRenderLayer,
        > + 'static,
{
    rudi::transient(|_| Box::new(P::default()) as Box<DesktopPlugin>)
        .name(std::any::type_name::<P>())
}

/// 将可默认构造的 desktop 插件注册到 Rudi 自动 provider 注册表。
#[macro_export]
macro_rules! register_desktop_plugin {
    ($plugin_ty:ty $(,)?) => {
        $crate::rudi::register_provider!($crate::desktop_plugin_provider::<$plugin_ty>());
    };
}

/// 声明一个可默认构造的 desktop 插件，并注册到 Rudi 自动 provider 注册表。
#[macro_export]
macro_rules! declare_desktop_plugin {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
    ) => {
        #[derive(Default)]
        $(#[$meta])*
        $vis struct $name;

        $crate::register_desktop_plugin!($name);
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($body:tt)*
        }
    ) => {
        #[derive(Default)]
        $(#[$meta])*
        $vis struct $name {
            $($body)*
        }

        $crate::register_desktop_plugin!($name);
    };
}
