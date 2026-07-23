use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};

/// 查看器能够自动识别的文件类型。
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FileViewerKind {
    Markdown,
    Pdf,
    Docx,
    Text,
    Unsupported,
}

impl FileViewerKind {
    /// 根据文件扩展名选择预览引擎。
    pub fn from_name(name: &str) -> Self {
        let extension = Path::new(name)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match extension.as_str() {
            "md" | "markdown" | "mdown" | "mkd" => Self::Markdown,
            "pdf" => Self::Pdf,
            "docx" => Self::Docx,
            "txt" | "log" | "json" | "jsonl" | "yaml" | "yml" | "toml" | "xml" | "csv" | "rs"
            | "kt" | "kts" | "java" | "js" | "jsx" | "ts" | "tsx" | "css" | "html" | "sql"
            | "sh" => Self::Text,
            _ => Self::Unsupported,
        }
    }

    pub(crate) fn engine_name(self) -> Option<&'static str> {
        match self {
            Self::Markdown => Some("markdown"),
            Self::Pdf => Some("pdf"),
            Self::Docx => Some("docx"),
            Self::Text => Some("text"),
            Self::Unsupported => None,
        }
    }

    pub(crate) fn extension_label(self, name: &str) -> String {
        let extension = Path::new(name)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("FILE");
        extension.to_ascii_uppercase()
    }

    fn accepts_text(self) -> bool {
        matches!(self, Self::Markdown | Self::Text)
    }

    fn mime(self, name: &str) -> String {
        match self {
            Self::Markdown => "text/markdown;charset=utf-8".to_string(),
            Self::Pdf => "application/pdf".to_string(),
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .to_string(),
            Self::Text => "text/plain;charset=utf-8".to_string(),
            Self::Unsupported => mime_guess::from_path(name)
                .first_or_octet_stream()
                .essence_str()
                .to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectedSource {
    Text,
    Blob,
    Url,
    Missing,
}

pub(crate) fn select_source(has_text: bool, has_blob: bool, has_url: bool) -> SelectedSource {
    if has_text {
        return SelectedSource::Text;
    }
    if has_blob {
        return SelectedSource::Blob;
    }
    if has_url {
        return SelectedSource::Url;
    }
    SelectedSource::Missing
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BlobLoad {
    NotRequested,
    Ready(Vec<u8>),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResolvedSource {
    Loading,
    Ready(String),
    Failed(String),
    Missing,
}

pub(crate) fn resolve_source(
    kind: FileViewerKind,
    name: &str,
    text: Option<&str>,
    has_blob: bool,
    src: Option<&str>,
    blob_load: Option<&BlobLoad>,
) -> ResolvedSource {
    let has_url = src.is_some_and(|value| !value.trim().is_empty());
    let selected = select_source(text.is_some(), has_blob, has_url);

    match selected {
        SelectedSource::Text => resolve_text(kind, name, text.unwrap_or_default()),
        SelectedSource::Blob => resolve_blob(kind, name, blob_load),
        SelectedSource::Url => ResolvedSource::Ready(src.unwrap_or_default().trim().to_string()),
        SelectedSource::Missing => ResolvedSource::Missing,
    }
}

fn resolve_text(kind: FileViewerKind, name: &str, text: &str) -> ResolvedSource {
    if !kind.accepts_text() {
        return ResolvedSource::Failed(format!(
            "{name} 是二进制格式，请使用 blob 或 src 传入文件内容"
        ));
    }
    ResolvedSource::Ready(data_url(kind, name, text.as_bytes()))
}

fn resolve_blob(kind: FileViewerKind, name: &str, blob_load: Option<&BlobLoad>) -> ResolvedSource {
    match blob_load {
        None => ResolvedSource::Loading,
        Some(BlobLoad::NotRequested) => ResolvedSource::Loading,
        Some(BlobLoad::Ready(bytes)) => ResolvedSource::Ready(data_url(kind, name, bytes)),
        Some(BlobLoad::Failed(message)) => ResolvedSource::Failed(message.clone()),
    }
}

fn data_url(kind: FileViewerKind, name: &str, bytes: &[u8]) -> String {
    let encoded = STANDARD.encode(bytes);
    format!("data:{};base64,{encoded}", kind.mime(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_formats() {
        assert_eq!(
            FileViewerKind::from_name("guide.md"),
            FileViewerKind::Markdown
        );
        assert_eq!(
            FileViewerKind::from_name("contract.PDF"),
            FileViewerKind::Pdf
        );
        assert_eq!(
            FileViewerKind::from_name("report.docx"),
            FileViewerKind::Docx
        );
        assert_eq!(
            FileViewerKind::from_name("events.log"),
            FileViewerKind::Text
        );
    }

    #[test]
    fn text_has_highest_source_priority() {
        assert_eq!(select_source(true, true, true), SelectedSource::Text);
        assert_eq!(select_source(false, true, true), SelectedSource::Blob);
        assert_eq!(select_source(false, false, true), SelectedSource::Url);
        assert_eq!(select_source(false, false, false), SelectedSource::Missing);
    }

    #[test]
    fn rejects_text_for_binary_formats() {
        let resolved = resolve_source(
            FileViewerKind::Pdf,
            "contract.pdf",
            Some("not a pdf"),
            false,
            None,
            None,
        );
        assert!(matches!(resolved, ResolvedSource::Failed(_)));
    }
}
