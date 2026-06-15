use az_aio_platform::plugin_api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn EdgeGatewayPage(context: NativeRenderContext) -> Element {
    rsx! {
        section { class: "native-plugin-page native-plugin-page--edge-gateway",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Operations / Network" }
                h1 { "Edge Gateway" }
                p { "Gateway flow templates, plan generation reference, runtime execution, and result panels." }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "Runtime" }
                    p { "Route: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/edge-gateway/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "Gateway API" }
                    ul {
                        li { "GET /api/edge-gateway/example" }
                        li { "POST /api/edge-gateway/run" }
                        li { "Toasty flow store with biz_edge_gateway_ prefix" }
                        li { "Rudi service context" }
                    }
                }
            }
        }
    }
}
