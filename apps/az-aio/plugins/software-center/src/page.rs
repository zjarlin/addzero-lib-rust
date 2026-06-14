use az_aio_platform::plugin_api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn SoftwareCenterPage(context: NativeRenderContext) -> Element {
    rsx! {
        section { class: "native-plugin-page native-plugin-page--software-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Knowledge / Software" }
                h1 { "Software Center" }
                p { "Installer scan, archive targets, and catalog-linked package detail surfaces." }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "Runtime" }
                    p { "Route: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/software-center/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "Installer Workflow" }
                    ul {
                        li { "GET /api/software-center/installers" }
                        li { "POST /api/software-center/organize" }
                        li { "Toasty package store with biz_software_center_ prefix" }
                        li { "shaku service module" }
                    }
                }
            }
        }
    }
}
