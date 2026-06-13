use az_aio_plugin_api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn AssetHubPage(context: NativeRenderContext) -> Element {
    rsx! {
        section { class: "native-plugin-page native-plugin-page--asset-hub",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Knowledge / Assets" }
                h1 { "Asset Hub" }
                p { "Assets, skill scan merge, compose assets, and subtype-backed metadata." }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "Runtime" }
                    p { "Route: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/asset-hub/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "Capabilities" }
                    ul {
                        li { "Dioxus content renderer" }
                        li { "Axum skill scan endpoints" }
                        li { "Toasty asset store with biz_asset_hub_ prefix" }
                        li { "shaku service module" }
                    }
                }
            }
        }
    }
}
