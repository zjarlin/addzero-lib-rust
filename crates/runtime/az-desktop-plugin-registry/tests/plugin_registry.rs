use az_desktop_plugin::api::{DesktopInitContext, DesktopRenderLayer, Plugin};
use az_desktop_plugin_registry::{api::load_plugins, declare_desktop_plugin};

declare_desktop_plugin! {
    struct AlphaPlugin;
}

impl
    Plugin<
        DesktopInitContext,
        az_desktop_plugin::api::DesktopEvent,
        az_desktop_plugin::api::DesktopExecContext,
        az_desktop_plugin::api::DesktopViewContext,
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

declare_desktop_plugin! {
    struct BetaPlugin;
}

impl
    Plugin<
        DesktopInitContext,
        az_desktop_plugin::api::DesktopEvent,
        az_desktop_plugin::api::DesktopExecContext,
        az_desktop_plugin::api::DesktopViewContext,
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
fn load_plugins_returns_registered_plugins_in_stable_name_order() {
    let plugins = load_plugins();
    let names = plugins
        .iter()
        .filter_map(|plugin| match plugin.name() {
            "alpha" | "beta" => Some(plugin.name()),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Registry order must remain stable so shell navigation does not jump between launches.
    assert_eq!(names, vec!["alpha", "beta"]);
}
