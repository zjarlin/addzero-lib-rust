//! Spring 风格环境变量占位符展开。

use std::env;

/// 展开字符串中的 Spring 风格环境变量占位符。
///
/// 支持 `${VAR}` 和 `${VAR:default}`。环境变量不存在或内容为空白时使用默认值；未闭合占位符会按原文本保留。
pub fn env_subst(input: impl AsRef<str>) -> String {
    let source = input.as_ref();
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0usize;

    while let Some(relative_start) = source[cursor..].find("${") {
        let start = cursor + relative_start;
        result.push_str(&source[cursor..start]);

        let placeholder = &source[start + 2..];
        if let Some(relative_end) = placeholder.find('}') {
            let end = start + 2 + relative_end;
            let body = &source[start + 2..end];
            let (name, default_value) = body.split_once(':').unwrap_or((body, ""));
            let value = env::var(name)
                .ok()
                .filter(|candidate| !candidate.trim().is_empty())
                .unwrap_or_else(|| default_value.to_owned());
            result.push_str(&value);
            cursor = end + 1;
        } else {
            result.push_str(&source[start..]);
            cursor = source.len();
            break;
        }
    }

    if cursor < source.len() {
        result.push_str(&source[cursor..]);
    }

    result
}
