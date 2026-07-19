use regex::Regex;
use std::sync::LazyLock;

static VERIFICATION_CODE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)verification code:?\s*(\d{6})",
        r"(?i)code is\s*(\d{6})",
        r">\s*(\d{6})\s*<",
        r"(?:^|[^#&[:digit:]])(\d{6})(?:[^[:digit:]]|$)",
    ]
    .into_iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

/// 从纯文本或 HTML 邮件内容中提取六位验证码。
///
/// 源项目跳过 `177010`，因为它是部分复制邮件模板中的已知布局误报；
/// 该辅助函数保留这个防护。
pub fn extract_verification_code(content: impl AsRef<str>) -> Option<String> {
    let content = content.as_ref();
    if content.trim().is_empty() {
        return None;
    }

    for pattern in VERIFICATION_CODE_PATTERNS.iter() {
        for capture in pattern.captures_iter(content) {
            let code = capture.get(1)?.as_str();
            if code != "177010" {
                return Some(code.to_owned());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_verification_code;

    #[test]
    fn extract_verification_code_prefers_explicit_phrase() {
        let content = "Your verification code: 123456";

        assert_eq!(
            extract_verification_code(content).as_deref(),
            Some("123456")
        );
    }

    #[test]
    fn extract_verification_code_skips_known_template_false_positive() {
        let content = "Template id 177010 and real code is 654321";

        assert_eq!(
            extract_verification_code(content).as_deref(),
            Some("654321")
        );
    }

    #[test]
    fn extract_verification_code_ignores_color_values() {
        let content = "background:#123456; code is 778899";

        assert_eq!(
            extract_verification_code(content).as_deref(),
            Some("778899")
        );
    }
}
