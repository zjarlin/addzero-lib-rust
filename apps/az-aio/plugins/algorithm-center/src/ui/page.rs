use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn AlgorithmCenterPage(context: NativeRenderContext) -> Element {
    let descriptors = az_algorithm::catalog::algorithm_component_descriptors();

    rsx! {
        section { class: "native-plugin-page native-plugin-page--algorithm-center",
            header { class: "native-plugin-page__header",
                p { class: "native-plugin-page__eyebrow", "Vision / Algorithms" }
                h1 { "算法中心" }
                p { "9 个视觉算法组件 · 目录浏览 · 输入输出契约" }
            }
            div { class: "native-plugin-page__grid",
                article { class: "native-plugin-card",
                    h2 { "运行时" }
                    p { "路由: {context.active_route}" }
                    p { "API: {context.api_base_url}/api/algorithm-center/status" }
                }
                article { class: "native-plugin-card",
                    h2 { "组件数: {descriptors.len()}" }
                    ul {
                        for descriptor in &descriptors {
                            li { "{descriptor.label}" }
                        }
                    }
                }
            }
        }
    }
}
