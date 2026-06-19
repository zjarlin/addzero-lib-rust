//! 面向 slug、路径标签和文件名 stem 的可复用文本清洗函数。

use std::path::{Component, Path};

use deunicode::deunicode;

/// 把文件系统路径转成使用 `/` 分隔的展示 key。
///
/// 该函数基于路径组件处理，而不是直接替换字符串，因此会先遵循宿主平台的路径分隔规则。
pub fn to_slash_path(path: impl AsRef<Path>) -> String {
    let mut output = String::new();
    for component in path.as_ref().components() {
        match component {
            Component::Prefix(prefix) => {
                append_path_segment(&mut output, &prefix.as_os_str().to_string_lossy());
            }
            Component::RootDir => {
                if output.is_empty() || !output.ends_with('/') {
                    output.push('/');
                }
            }
            Component::CurDir => append_path_segment(&mut output, "."),
            Component::ParentDir => append_path_segment(&mut output, ".."),
            Component::Normal(part) => {
                append_path_segment(&mut output, &part.to_string_lossy());
            }
        }
    }
    output
}

/// 清洗 URL 或文件路径片段，并保留扩展名中的点号。
///
/// ASCII 字母、数字、`.`、`-` 和 `_` 会被保留；其他字符会变成 `-`，
/// 最后移除首尾多余的 `-`。
pub fn sanitize_path_segment(input: &str) -> String {
    replace_disallowed_ascii(input, ".-_", '-', true)
}

/// 替换 ASCII 字母、数字和 `extra_allowed` 以外的字符。
///
/// 这是协议专用标签的底层原语：允许调用方自定义额外可接受字符，
/// 同时复用同一套 ASCII 边界规则。
pub fn sanitize_ascii_label(input: &str, extra_allowed: &str, replacement: char) -> String {
    replace_disallowed_ascii(input, extra_allowed, replacement, false)
}

/// 替换 ASCII 字母、数字和 `extra_allowed` 以外的字符；结果为空时返回 `fallback`。
pub fn sanitize_ascii_label_or(
    input: &str,
    extra_allowed: &str,
    replacement: char,
    fallback: &str,
) -> String {
    let sanitized = sanitize_ascii_label(input, extra_allowed, replacement);
    if sanitized.is_empty() {
        fallback.to_owned()
    } else {
        sanitized
    }
}

/// 清洗单个文件 stem，并保留可读分隔符。
///
/// ASCII 字母、数字、`-` 和 `_` 会被保留；其他字符会变成 `_`。
pub fn sanitize_file_stem(input: &str) -> String {
    sanitize_ascii_label(input, "-_", '_')
}

/// 清洗文件 stem；清洗结果为空时返回 `fallback`。
pub fn sanitize_file_stem_or(input: &str, fallback: &str) -> String {
    let sanitized = sanitize_file_stem(input);
    if sanitized.is_empty() {
        fallback.to_owned()
    } else {
        sanitized
    }
}

/// 取路径的 file stem 并清洗；缺失或清洗结果为空时返回 `fallback`。
pub fn sanitize_path_file_stem_or(path: impl AsRef<Path>, fallback: &str) -> String {
    let stem = path
        .as_ref()
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    sanitize_file_stem_or(stem, fallback)
}

/// 清洗文件名并确保带有指定扩展名。
///
/// 该函数适合认证文件、缓存文件等“需要可读标签但不能允许路径分隔符”的场景。
/// `extension` 可传 `json` 或 `.json`；清洗后的主体为空时使用 `fallback`。
pub fn sanitize_file_name_with_extension(
    input: &str,
    extra_allowed: &str,
    replacement: char,
    extension: &str,
    fallback: &str,
) -> String {
    let mut stem = sanitize_ascii_label(input, extra_allowed, replacement);
    if stem.trim_matches(replacement).is_empty() {
        stem = fallback.to_owned();
    }

    let extension = extension.trim_start_matches('.');
    if extension.is_empty() {
        return stem;
    }

    let suffix = format!(".{extension}");
    if !stem.ends_with(&suffix) {
        stem.push_str(&suffix);
    }
    stem
}

/// 只保留 ASCII 字母和数字。
pub fn ascii_alphanumeric(input: &str) -> String {
    input.chars().filter(char::is_ascii_alphanumeric).collect()
}

/// 只保留 ASCII 字母和数字；结果为空时返回 `fallback`。
pub fn ascii_alphanumeric_or(input: &str, fallback: &str) -> String {
    let sanitized = ascii_alphanumeric(input);
    if sanitized.is_empty() {
        fallback.to_owned()
    } else {
        sanitized
    }
}

/// 把任意文本转成稳定的小写 ASCII slug。
///
/// Unicode 文本会先经 `deunicode` 转写；非字母数字片段会压缩为单个 `-`，
/// 最后移除首尾多余的 `-`。
pub fn to_slug(input: &str) -> String {
    let normalized = deunicode(input);
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in normalized.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_owned()
}

/// 把文本转成 slug；结果为空时返回 `fallback`。
pub fn to_slug_or(input: &str, fallback: &str) -> String {
    let slug = to_slug(input);
    if slug.is_empty() {
        fallback.to_owned()
    } else {
        slug
    }
}

/// 把 slug 风格文本转成首字母大写的单词标题。
pub fn title_case_slug(input: &str) -> String {
    input
        .split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut word = first.to_uppercase().collect::<String>();
            word.push_str(chars.as_str());
            word
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn replace_disallowed_ascii(
    input: &str,
    extra_allowed: &str,
    replacement: char,
    trim_replacement: bool,
) -> String {
    let value = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || extra_allowed.contains(ch) {
                ch
            } else {
                replacement
            }
        })
        .collect::<String>();

    if trim_replacement {
        value.trim_matches(replacement).to_owned()
    } else {
        value
    }
}

fn append_path_segment(output: &mut String, segment: &str) {
    if !output.is_empty() && !output.ends_with('/') {
        output.push('/');
    }
    output.push_str(&segment.replace('\\', "/"));
}
