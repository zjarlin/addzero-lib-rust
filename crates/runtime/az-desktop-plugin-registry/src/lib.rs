#![forbid(unsafe_code)]

use az_desktop_plugin::DesktopPlugin;

pub use inventory;

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

#[cfg(test)]
mod tests {
    use az_desktop_plugin::{DesktopInitContext, DesktopRenderLayer, Plugin};

    use super::load_plugins;

    struct AlphaPlugin;

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

    struct BetaPlugin;

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

    inventory::submit! {
        super::DesktopPluginRegistration {
            constructor: || Box::new(BetaPlugin),
        }
    }

    inventory::submit! {
        super::DesktopPluginRegistration {
            constructor: || Box::new(AlphaPlugin),
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
