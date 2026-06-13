use az_aio_plugin_api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn DriveCenterPage(context: NativeRenderContext) -> Element {
    rsx! {
        section { class: "native-plugin-page native-plugin-page--drive-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Operations / Storage" }
                h1 { "Drive Center" }
                p { "Drive hosting, tracked items, queue, conflicts, and root aliases." }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "Runtime" }
                    p { "Route: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/drive-center/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "Queue" }
                    ul {
                        li { "GET /api/drive-center/tasks" }
                        li { "POST /api/drive-center/task" }
                        li { "Toasty task store with biz_drive_center_ prefix" }
                        li { "shaku service module" }
                    }
                }
            }
        }
    }
}
