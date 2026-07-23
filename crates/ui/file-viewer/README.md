# Dioxus 文件预览

`az-file-viewer` 提供统一的 Dioxus `FileViewer` 组件，当前支持 Markdown、PDF、DOCX 和普通文本。浏览器运行时资源已经包含在 crate 中，业务项目不需要安装 npm 依赖。

## 使用

```toml
[dependencies]
az-file-viewer = { path = "crates/ui/file-viewer" }
```

```rust
use az_file_viewer::FileViewer;
use dioxus::prelude::*;

fn Preview(markdown_text: String, selected_file: FileData, file_url: String) -> Element {
    rsx! {
        FileViewer {
            text: markdown_text,
            name: "说明.md"
        }
        FileViewer {
            blob: selected_file.clone(),
            name: selected_file.name()
        }
        FileViewer {
            src: file_url,
            name: "合同.pdf"
        }
    }
}
```

输入优先级为 `text > blob > src`：

- `text`：已解码 Markdown 或普通文本。
- `blob`：Dioxus 文件选择事件返回的 `FileData`。
- `src`：同源或允许 CORS 的文件 URL。

PDF、DOCX 与本地二进制文件会在浏览器/WebView 内解析，不会上传到第三方服务。
