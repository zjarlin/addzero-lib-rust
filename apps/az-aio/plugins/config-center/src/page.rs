use az_aio_plugin_api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn ConfigCenterPage(context: NativeRenderContext) -> Element {
    rsx! {
        section { class: "native-plugin-page native-plugin-page--config-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Environment / Machine" }
                h1 { "Config Center" }
                p { "Dotfiles monitor, pairing identity, XDG paths, and provider configuration." }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "Runtime" }
                    p { "Route: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/config-center/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "Data Boundaries" }
                    ul {
                        li { "Toasty config store with biz_config_center_ prefix" }
                        li { "Dotfiles monitor API" }
                        li { "Local pairing identity API" }
                        li { "shaku service module" }
                    }
                }
            }
        }
    }
}
