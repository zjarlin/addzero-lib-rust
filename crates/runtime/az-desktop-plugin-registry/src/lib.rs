#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use az_derive_aliases::{apply, plain_copy};
use az_desktop_plugin::{
    DesktopEvent, DesktopExecContext, DesktopInitContext, DesktopPlugin, DesktopRenderLayer,
    DesktopViewContext, Plugin,
};

#[doc(hidden)]
pub use az_derive_aliases as __az_desktop_plugin_registry_derive_aliases;
pub use inventory;

#[apply(plain_copy)]
pub struct DesktopPluginRegistration {
    pub constructor: fn() -> Box<DesktopPlugin>,
}

inventory::collect!(DesktopPluginRegistration);

pub fn load_plugins() -> Vec<Box<DesktopPlugin>> {
    let mut plugins: Vec<_> = inventory::iter::<DesktopPluginRegistration>
        .into_iter()
        .map(|registration| (registration.constructor)())
        .collect();
    plugins.sort_by(|left, right| left.name().cmp(right.name()));
    plugins
}

/// Builds a default-constructible desktop plugin as a boxed plugin trait object.
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

/// Registers a default-constructible desktop plugin in the distributed registry.
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

/// Declares a default-constructible desktop plugin and registers it in the distributed registry.
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

#[cfg(test)]
mod tests {
    use az_desktop_plugin::{DesktopInitContext, DesktopRenderLayer, Plugin};

    use super::load_plugins;

    crate::declare_desktop_plugin! {
        struct AlphaPlugin;
    }

    impl
        Plugin<
            DesktopInitContext,
            az_desktop_plugin::DesktopEvent,
            az_desktop_plugin::DesktopExecContext,
            az_desktop_plugin::DesktopViewContext,
            DesktopRenderLayer,
        > for AlphaPlugin
    {
        fn name(&self) -> &'static str {
            "alpha"
        }

        fn render_layer(&self) -> DesktopRenderLayer {
            DesktopRenderLayer::Main
        }
    }

    crate::declare_desktop_plugin! {
        struct BetaPlugin;
    }

    impl
        Plugin<
            DesktopInitContext,
            az_desktop_plugin::DesktopEvent,
            az_desktop_plugin::DesktopExecContext,
            az_desktop_plugin::DesktopViewContext,
            DesktopRenderLayer,
        > for BetaPlugin
    {
        fn name(&self) -> &'static str {
            "beta"
        }

        fn render_layer(&self) -> DesktopRenderLayer {
            DesktopRenderLayer::Overlay
        }
    }

    #[test]
    fn loads_plugins_sorted_by_name() {
        let plugins = load_plugins();
        let names = plugins
            .iter()
            .filter_map(|plugin| match plugin.name() {
                "alpha" | "beta" => Some(plugin.name()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}
