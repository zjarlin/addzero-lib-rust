#![allow(non_snake_case)]

//! 当前客户机 Rust 代码生成页面。

use az_aio_platform::plugin::contract::NativeRenderContext;
use dioxus::prelude::*;
use registry::ui::{
    button::Button,
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
};

use crate::contract::UI_ACTION_PATH;

/// 渲染 enum 和 struct 客户机生成表单。
pub fn CodegenPage(context: NativeRenderContext) -> Element {
    let generated = parse_query_param(&context.active_route, "generated");
    let error = parse_query_param(&context.active_route, "error");
    let default_directory = ".";

    rsx! {
        div { class: "space-y-6",
            Card {
                CardHeader {
                    CardTitle { "Rust 代码生成" }
                    CardDescription { "生成操作在运行当前 AIO native backend 的客户机执行；写接口只接受本机回环请求，浏览器只提交结构化定义和目标目录。" }
                }
                CardContent { class: "flex flex-wrap gap-2 text-sm text-muted-foreground",
                    span { class: "rounded-md border px-2 py-1", "执行节点：当前客户机" }
                    span { class: "rounded-md border px-2 py-1", "默认不覆盖已有文件" }
                    span { class: "rounded-md border px-2 py-1", "输出：格式化 Rust 源码" }
                }
            }

            if let Some(path) = generated {
                div { class: "rounded-xl border border-green-600/40 bg-green-600/10 p-4 text-sm",
                    strong { "生成完成：" }
                    code { "{path}" }
                }
            }
            if let Some(message) = error {
                div { class: "rounded-xl border border-destructive bg-destructive/10 p-4 text-sm text-destructive",
                    "{message}"
                }
            }

            div { class: "grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(300px,420px)]",
                Card {
                    CardHeader {
                        CardTitle { "生成设置" }
                        CardDescription { "目录可以使用绝对路径、相对客户机授权根目录的路径，或 ~/ 开头的路径；默认授权根目录为 $HOME。" }
                    }
                    CardContent {
                        form { method: "post", action: UI_ACTION_PATH, class: "grid gap-4",
                            FieldLabel { title: "客户机目标目录", required: true,
                                input {
                                    class: "aio-input",
                                    name: "target_directory",
                                    value: default_directory,
                                    required: true,
                                }
                            }
                            div { class: "grid gap-4 md:grid-cols-2",
                                FieldLabel { title: "类型种类", required: true,
                                    select { class: "aio-input", name: "type_kind", required: true,
                                        option { value: "enum", "enum" }
                                        option { value: "struct", "struct" }
                                    }
                                }
                                FieldLabel { title: "类型名", required: true,
                                    input { class: "aio-input", name: "type_name", placeholder: "DeviceState", required: true }
                                }
                            }
                            FieldLabel { title: "文件名（留空自动生成）",
                                input { class: "aio-input", name: "file_name", placeholder: "device_state.rs" }
                            }
                            label { class: "grid gap-2 text-sm",
                                span { class: "font-medium", "成员定义" }
                                textarea {
                                    class: "aio-input min-h-48 font-mono",
                                    name: "members",
                                    required: true,
                                    placeholder: "Pending\nRunning = 1\nFailed = 2",
                                }
                            }
                            label { class: "flex items-center gap-2 text-sm",
                                input { r#type: "checkbox", name: "overwrite", value: "true" }
                                span { "允许覆盖客户机上的同名 .rs 文件" }
                            }
                            div { class: "flex justify-end",
                                Button { button_type: "submit", "生成到客户机" }
                            }
                        }
                    }
                }

                div { class: "grid gap-4",
                    ExampleCard {
                        title: "enum 成员".to_string(),
                        description: "每行一个变体，可使用 = 整数设置判别值。".to_string(),
                        source: "Pending\nRunning = 1\nFailed = 2".to_string(),
                    }
                    ExampleCard {
                        title: "struct 字段".to_string(),
                        description: "每行使用 字段名: Rust类型，可填写 Option、Vec 等完整类型。".to_string(),
                        source: "device_id: String\nonline: bool\ntags: Vec<String>".to_string(),
                    }
                }
            }
        }
    }
}

#[component]
fn FieldLabel(title: String, #[props(default)] required: bool, children: Element) -> Element {
    rsx! {
        label { class: "grid gap-2 text-sm",
            span { class: "font-medium",
                "{title}"
                if required { span { class: "text-destructive", " *" } }
            }
            {children}
        }
    }
}

#[component]
fn ExampleCard(title: String, description: String, source: String) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { "{description}" }
            }
            CardContent {
                pre { class: "overflow-x-auto rounded-lg bg-muted p-4 text-sm",
                    code { "{source}" }
                }
            }
        }
    }
}

fn parse_query_param(route: &str, key: &str) -> Option<String> {
    let query = route.split_once('?')?.1;
    for pair in query.split('&') {
        let (pair_key, pair_value) = pair.split_once('=').unwrap_or((pair, ""));
        if pair_key != key {
            continue;
        }
        return Some(
            urlencoding::decode(pair_value)
                .map(|value| value.into_owned())
                .unwrap_or_else(|_| pair_value.to_string()),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_generated_client_path_from_route() {
        let path = parse_query_param(
            "/codegen?generated=%2Ftmp%2Fgenerated%2Fdevice.rs",
            "generated",
        );

        // 关键断言：SSR 重定向后必须展示客户机实际写入路径。
        assert_eq!(path.as_deref(), Some("/tmp/generated/device.rs"));
    }
}
