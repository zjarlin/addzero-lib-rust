use regex::Regex;
use std::sync::LazyLock;

static VERIFICATION_CODE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r#"class=["']code["'][^>]*>\s*(\d{6})\s*<"#,
        r"(?i)verification\s+code[:\s]*(\d{6})",
        r"(?i)code\s+is\s*(\d{6})",
        r"(?:^|[^\d])(\d{6})(?:[^\d]|$)",
    ]
    .into_iter()
    .map(|pattern| Regex::new(pattern).expect("verification-code regex should compile"))
    .collect()
});

/// 从纯文本或 HTML 邮件内容中提取六位验证码。
#[must_use]
pub fn extract_verification_code(content: impl AsRef<str>) -> Option<String> {
    let content = content.as_ref();
    if content.trim().is_empty() {
        return None;
    }

    for pattern in VERIFICATION_CODE_PATTERNS.iter() {
        for capture in pattern.captures_iter(content) {
            return capture.get(1).map(|code| code.as_str().to_owned());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_verification_code;

    #[test]
    fn extracts_code_from_emailnator_html_class() {
        assert_eq!(
            extract_verification_code(r#"<div class="code"> 123456 </div>"#).as_deref(),
            Some("123456")
        );
    }

    #[test]
    fn extracts_code_from_phrase() {
        assert_eq!(
            extract_verification_code("Verification code: 654321").as_deref(),
            Some("654321")
        );
    }
}
