use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use dioxus::prelude::{dioxus_elements::FileData, *};

use crate::{
    assets::{
        DOCX_SCRIPT, DOMPURIFY_SCRIPT, JSZIP_SCRIPT, MARKED_SCRIPT, PDF_MODULE, PDF_WORKER,
        VIEWER_ENGINE, VIEWER_STYLE,
    },
    source::{BlobLoad, FileViewerKind, ResolvedSource, resolve_source},
};

/// 查看器主题。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileViewerTheme {
    Light,
    Dark,
    #[default]
    System,
}

impl FileViewerTheme {
    fn attribute(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

/// `FileViewer` 的统一输入参数。
///
/// 输入优先级固定为 `text > blob > src`。`blob` 使用 Dioxus 文件选择事件返回的
/// [`FileData`]，因此 Web、Desktop 和 Mobile 调用方共享同一个组件 API。
#[derive(Clone, PartialEq, Props)]
pub struct FileViewerProps {
    /// 文件名，用于标题展示和预览类型识别。
    #[props(into)]
    pub name: String,
    /// 已解码文本，适用于 Markdown 和普通文本。
    #[props(into, default)]
    pub text: Option<String>,
    /// Dioxus 文件选择事件返回的本地文件。
    #[props(into, default)]
    pub blob: Option<FileData>,
    /// 可由当前 WebView 访问的文件 URL。
    #[props(into, default)]
    pub src: Option<String>,
    /// 查看器主题。
    #[props(default)]
    pub theme: FileViewerTheme,
    /// 追加到查看器根节点的 CSS 类名。
    #[props(into, default)]
    pub class: Option<String>,
}

/// 在统一面板中预览 Markdown、PDF、DOCX 和普通文本。
///
/// PDF、DOCX 和通过 URL 加载的文本由 crate 内置浏览器资源处理，调用方不需要安装 npm 依赖。
pub fn FileViewer(props: FileViewerProps) -> Element {
    let detected_kind = FileViewerKind::from_name(&props.name);
    let kind = if detected_kind == FileViewerKind::Unsupported && props.text.is_some() {
        FileViewerKind::Text
    } else {
        detected_kind
    };
    let blob_for_loading = if props.text.is_none() {
        props.blob.clone()
    } else {
        None
    };
    let blob_resource = use_resource(use_reactive!(|(blob_for_loading,)| async move {
        let Some(file) = blob_for_loading else {
            return BlobLoad::NotRequested;
        };
        match file.read_bytes().await {
            Ok(bytes) => BlobLoad::Ready(bytes.to_vec()),
            Err(error) => BlobLoad::Failed(format!("读取本地文件失败：{error}")),
        }
    }));
    let blob_state = blob_resource.read();
    let resolved = resolve_source(
        kind,
        &props.name,
        props.text.as_deref(),
        props.blob.is_some(),
        props.src.as_deref(),
        blob_state.as_ref(),
    );
    let extension = kind.extension_label(&props.name);
    let root_class = props
        .class
        .as_deref()
        .map(|class| format!("file-viewer {class}"))
        .unwrap_or_else(|| "file-viewer".to_string());
    let body = render_body(kind, &props.name, &resolved);
    let download = match &resolved {
        ResolvedSource::Ready(url) => Some(url.clone()),
        _ => None,
    };

    rsx! {
        document::Stylesheet { href: VIEWER_STYLE }
        document::Script { src: VIEWER_ENGINE }
        article {
            class: root_class,
            "data-theme": props.theme.attribute(),
            header { class: "file-viewer__header",
                div { class: "file-viewer__title", title: props.name.clone(),
                    span { class: "file-viewer__name", "{props.name}" }
                    span { class: "file-viewer__extension", "{extension}" }
                }
                if let Some(url) = download {
                    a {
                        class: "file-viewer__action",
                        href: url,
                        download: props.name.clone(),
                        title: "下载原文件",
                        "下载"
                    }
                }
            }
            section { class: "file-viewer__body", {body} }
        }
    }
}

fn render_body(kind: FileViewerKind, name: &str, resolved: &ResolvedSource) -> Element {
    match resolved {
        ResolvedSource::Loading => rsx! {
            p { class: "file-viewer__message", "正在读取文件…" }
        },
        ResolvedSource::Failed(message) => rsx! {
            p { class: "file-viewer__message file-viewer__message--error", "{message}" }
        },
        ResolvedSource::Missing => rsx! {
            p { class: "file-viewer__message", "未提供文件内容" }
        },
        ResolvedSource::Ready(url) => render_ready(kind, name, url),
    }
}

fn render_ready(kind: FileViewerKind, name: &str, url: &str) -> Element {
    let Some(engine) = kind.engine_name() else {
        return rsx! {
            div { class: "file-viewer__unsupported",
                p { "当前版本暂不支持直接预览此格式" }
                a { href: url, download: name, "下载原文件" }
            }
        };
    };
    let runtime_key = runtime_key(kind, name, url);

    rsx! {
        div {
            key: "{runtime_key}",
            class: "file-viewer__runtime",
            "data-file-viewer-engine": "true",
            "data-kind": engine,
            "data-name": name,
            "data-src": url,
            "data-marked-script": MARKED_SCRIPT.to_string(),
            "data-dompurify-script": DOMPURIFY_SCRIPT.to_string(),
            "data-pdf-module": PDF_MODULE.to_string(),
            "data-pdf-worker": PDF_WORKER.to_string(),
            "data-jszip-script": JSZIP_SCRIPT.to_string(),
            "data-docx-script": DOCX_SCRIPT.to_string(),
            p { class: "file-viewer__message", "正在准备预览…" }
        }
    }
}

fn runtime_key(kind: FileViewerKind, name: &str, url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    (kind, name, url).hash(&mut hasher);
    format!("file-viewer-{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown_runtime_contract() {
        let html = dioxus_ssr::render_element(rsx! {
            FileViewer {
                name: "说明.md",
                text: "# 标题\n\n正文"
            }
        });
        assert!(html.contains("data-kind=\"markdown\""));
        assert!(html.contains("说明.md"));
        assert!(html.contains("file-viewer__runtime"));
    }

    #[test]
    fn runtime_key_changes_with_source() {
        let first = runtime_key(
            FileViewerKind::Pdf,
            "contract.pdf",
            "data:application/pdf,a",
        );
        let second = runtime_key(
            FileViewerKind::Pdf,
            "contract.pdf",
            "data:application/pdf,b",
        );

        assert_ne!(first, second);
    }
}
