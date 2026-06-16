//! Reusable text sanitizers for slugs, path labels, and file-name stems.

use std::path::{Component, Path};

use deunicode::deunicode;

/// Converts a filesystem path to a forward-slash separated display key.
///
/// The function uses path components instead of raw string replacement, so it
/// follows the host platform's separator rules before joining with `/`.
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

/// Sanitizes a URL/file path segment while preserving extensions.
///
/// ASCII letters, digits, `.`, `-`, and `_` are preserved. Every other
/// character is converted to `-`, then leading and trailing `-` are removed.
pub fn sanitize_path_segment(input: &str) -> String {
    replace_disallowed_ascii(input, ".-_", '-', true)
}

/// Replaces characters outside ASCII letters, digits, and `extra_allowed`.
///
/// This is the low-level primitive for protocol-specific labels that need a
/// custom allowed-character set but still want the same ASCII boundary rule.
pub fn sanitize_ascii_label(input: &str, extra_allowed: &str, replacement: char) -> String {
    replace_disallowed_ascii(input, extra_allowed, replacement, false)
}

/// Sanitizes a single file stem while preserving readable separators.
///
/// ASCII letters, digits, `-`, and `_` are preserved. Every other character is
/// converted to `_`.
pub fn sanitize_file_stem(input: &str) -> String {
    sanitize_ascii_label(input, "-_", '_')
}

/// Sanitizes a file stem and returns `fallback` when the result is empty.
pub fn sanitize_file_stem_or(input: &str, fallback: &str) -> String {
    let sanitized = sanitize_file_stem(input);
    if sanitized.is_empty() {
        fallback.to_owned()
    } else {
        sanitized
    }
}

/// Keeps only ASCII letters and digits.
pub fn ascii_alphanumeric(input: &str) -> String {
    input
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Converts arbitrary text into a stable lowercase ASCII slug.
///
/// Unicode text is transliterated with `deunicode`; non-alphanumeric runs are
/// collapsed to a single `-`, and leading/trailing `-` are removed.
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

/// Converts text into a slug and returns `fallback` when the slug is empty.
pub fn to_slug_or(input: &str, fallback: &str) -> String {
    let slug = to_slug(input);
    if slug.is_empty() {
        fallback.to_owned()
    } else {
        slug
    }
}

/// Converts a slug-like value into title case words.
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
