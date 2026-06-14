#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use az_derive_aliases::{apply, plain_copy};
use az_desktop_plugin::api::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopPlugin, DesktopRenderLayer,
    DesktopViewContext, Plugin,
};

#[doc(hidden)]

/// 分布式 desktop 插件注册项。
#[apply(plain_copy)]
pub struct DesktopPluginRegistration {
    /// 构造 boxed desktop plugin 的函数指针。
    pub constructor: fn() -> Box<DesktopPlugin>,
}

inventory::collect!(DesktopPluginRegistration);

/// 从 `inventory` 分布式注册表加载全部 desktop 插件。
///
/// 返回列表按插件名排序，保证宿主 shell 每次启动时顺序稳定。
pub fn load_plugins() -> Vec<Box<DesktopPlugin>> {
    let mut plugins: Vec<_> = inventory::iter::<DesktopPluginRegistration>
        .into_iter()
        .map(|registration| (registration.constructor)())
        .collect();
    plugins.sort_by(|left, right| left.name().cmp(right.name()));
    plugins
}

/// 将可默认构造的 desktop 插件包装成 boxed trait object。
pub fn default_desktop_plugin_constructor<P>() -> Box<DesktopPlugin>
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
    Box::new(P::default())
}

/// 将可默认构造的 desktop 插件注册到分布式注册表。
#[macro_export]
macro_rules! register_desktop_plugin {
    ($plugin_ty:ty $(,)?) => {
        $crate::inventory::submit! {
            $crate::DesktopPluginRegistration {
                constructor: $crate::default_desktop_plugin_constructor::<$plugin_ty>,
            }
        }
    };
}

/// 声明一个可默认构造的 desktop 插件，并注册到分布式注册表。
#[macro_export]
macro_rules! declare_desktop_plugin {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $crate::__az_desktop_plugin_registry_derive_aliases::plain_default! {
            $(#[$meta])*
            $vis struct $name;
        }

        $crate::register_desktop_plugin!($name);
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_desktop_plugin_registry_derive_aliases::plain_default! {
            $(#[$meta])*
            $vis struct $name {
                $($body)*
            }
        }

        $crate::register_desktop_plugin!($name);
    };
}
