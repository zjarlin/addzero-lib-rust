#![forbid(unsafe_code)]
#![allow(non_snake_case)]

use az_file_viewer::FileViewer;
use dioxus::prelude::{dioxus_elements::FileData, *};

const SAMPLE_MARKDOWN: &str = r#"# 电力调度方案 SSE 前端联调说明

`response` 是 `/admin-api/ai/power/plan-design/chat-stream` 的成功流示例。

## 前端处理规则

1. 逐段读取 `data:` 后面的 JSON。
2. `type=tool_result` 时解析 `toolOutput`。
3. `type=done` 且 `done=true` 表示正常结束。

```json
{"contextModes":["knowledge"]}
```
"#;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let mut selected_file = use_signal(|| None::<FileData>);
    let mut remote_name = use_signal(|| "合同.pdf".to_string());
    let mut remote_url = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: asset!("/examples/demo.css") }
        main { class: "demo-shell",
            header { class: "demo-toolbar",
                div {
                    h1 { "Dioxus FileViewer" }
                    p { "text、blob、src 三种输入使用同一个组件" }
                }
                label { class: "demo-file-picker",
                    span { "选择 Markdown / PDF / DOCX" }
                    input {
                        r#type: "file",
                        accept: ".md,.markdown,.txt,.pdf,.docx",
                        onchange: move |event| {
                            selected_file.set(event.files().into_iter().next());
                        }
                    }
                }
            }
            section { class: "demo-grid",
                div { class: "demo-preview",
                    if let Some(file) = selected_file() {
                        FileViewer { name: file.name(), blob: file }
                    } else {
                        FileViewer { name: "response说明.md", text: SAMPLE_MARKDOWN }
                    }
                }
                aside { class: "demo-controls",
                    h2 { "URL 预览" }
                    label {
                        span { "文件名" }
                        input {
                            value: remote_name,
                            oninput: move |event| remote_name.set(event.value())
                        }
                    }
                    label {
                        span { "文件 URL" }
                        input {
                            value: remote_url,
                            placeholder: "https://example.com/contract.pdf",
                            oninput: move |event| remote_url.set(event.value())
                        }
                    }
                    if !remote_url().trim().is_empty() {
                        div { class: "demo-remote-preview",
                            FileViewer { name: remote_name(), src: remote_url() }
                        }
                    }
                }
            }
        }
    }
}
