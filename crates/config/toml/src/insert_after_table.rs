//! TOML 文本插入辅助函数。

/// 在 TOML 文本指定表头后插入内容。
///
/// `tag` 可以是 `plugins` 或 `[plugins]`。找不到表头或表头行没有换行时返回原文本，不尝试解析或重排 TOML。
pub fn insert_after_table(content: &str, tag: &str, append_text: &str) -> String {
    let normalized_tag = if tag.starts_with('[') {
        tag.to_owned()
    } else {
        format!("[{tag}]")
    };

    let Some(tag_index) = content.find(&normalized_tag) else {
        return content.to_owned();
    };

    let Some(relative_newline) = content[tag_index..].find('\n') else {
        return content.to_owned();
    };

    let insert_at = tag_index + relative_newline + 1;
    let mut result = String::with_capacity(content.len() + append_text.len() + 1);
    result.push_str(&content[..insert_at]);
    result.push_str(append_text);
    result.push('\n');
    result.push_str(&content[insert_at..]);
    result
}
