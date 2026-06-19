use crate::model::{GmailMessage, GmailMessagePart};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

static DEFAULT_CODE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r#"class=["'](?:code|otp|verification-code)["'][^>]*>\s*([0-9][0-9\s-]{2,14}[0-9])\s*<"#,
        r"(?i)(?:verification|security|login|one[-\s]?time|auth(?:entication)?|确认|验证|验证码|校验码)\s*(?:code|码)?[:：\s]*(\d[\d\s-]{2,14}\d)",
        r"(?i)(?:code|otp|pin)\s*(?:is|为|是)?[:：\s]*(\d[\d\s-]{2,14}\d)",
        r"(?:^|[^#&[:alnum:]])(\d[\d\s-]{2,14}\d)(?:[^[:alnum:]]|$)",
    ]
    .into_iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

/// 控制数字验证码提取范围的选项。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtractCodeOptions {
    /// 移除分隔符后的最小数字位数。
    pub min_digits: usize,
    /// 移除分隔符后的最大数字位数。
    pub max_digits: usize,
}

impl ExtractCodeOptions {
    /// 创建提取选项；取值会被归一化到实用 OTP 范围。
    #[must_use]
    pub const fn new(min_digits: usize, max_digits: usize) -> Self {
        let min_digits = if min_digits < 3 { 3 } else { min_digits };
        let max_digits = if max_digits < min_digits {
            min_digits
        } else {
            max_digits
        };
        Self {
            min_digits,
            max_digits,
        }
    }
}

impl Default for ExtractCodeOptions {
    fn default() -> Self {
        ExtractCodeOptions {
    min_digits: 4,
    max_digits: 8,
}
    }
}

/// 已解码、待检查验证码的邮件正文候选。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageBodyCandidate {
    /// Gmail 正文 part 的 MIME 类型。
    pub mime_type: String,
    /// 解码后的文本内容。
    pub text: String,
}

/// 从纯文本或 HTML 中提取 4 到 8 位数字验证码。
#[must_use]
pub fn extract_verification_code(content: impl AsRef<str>) -> Option<String> {
    extract_verification_code_with_options(content, ExtractCodeOptions::default())
}

/// 使用自定义数字长度选项提取数字验证码。
#[must_use]
pub fn extract_verification_code_with_options(
    content: impl AsRef<str>,
    options: ExtractCodeOptions,
) -> Option<String> {
    let content = content.as_ref();
    if content.trim().is_empty() {
        return None;
    }

    let normalized = html_to_searchable_text(content);
    for pattern in DEFAULT_CODE_PATTERNS.iter() {
        for capture in pattern.captures_iter(&normalized) {
            let Some(raw) = capture.get(1).map(|code| code.as_str()) else {
                continue;
            };
            let code = normalize_code(raw);
            if code.len() >= options.min_digits
                && code.len() <= options.max_digits
                && !looks_like_false_positive(&code)
            {
                return Some(code);
            }
        }
    }
    None
}

/// 从 Gmail 邮件中收集已解码的 `text/plain` 和 `text/html` 正文候选。
pub fn collect_message_body_candidates(
    message: &GmailMessage,
) -> anyhow::Result<Vec<MessageBodyCandidate>> {
    let mut plain = Vec::new();
    let mut html = Vec::new();

    if let Some(payload) = &message.payload {
        collect_part_candidates(payload, "root", &mut plain, &mut html)?;
    }

    if plain.is_empty() {
        plain = html;
    }

    if plain.is_empty()
        && let Some(snippet) = message
            .snippet
            .as_ref()
            .filter(|value| !value.trim().is_empty())
    {
        plain.push(MessageBodyCandidate {
            mime_type: "snippet".to_owned(),
            text: snippet.clone(),
        });
    }

    Ok(plain)
}

fn collect_part_candidates(
    part: &GmailMessagePart,
    fallback_id: &str,
    plain: &mut Vec<MessageBodyCandidate>,
    html: &mut Vec<MessageBodyCandidate>,
) -> anyhow::Result<()> {
    let part_id = part.part_id.as_deref().unwrap_or(fallback_id);
    if let Some(data) = part.body.as_ref().and_then(|body| body.data.as_deref()) {
        let decoded = decode_gmail_body(data, part_id)?;
        let candidate = MessageBodyCandidate {
            mime_type: part.mime_type.clone(),
            text: decoded,
        };
        match part.mime_type.as_str() {
            "text/plain" => plain.push(candidate),
            "text/html" => html.push(candidate),
            _ => {}
        }
    }

    for (index, child) in part.parts.iter().enumerate() {
        let child_id = child
            .part_id
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(format!("{part_id}.{index}")));
        collect_part_candidates(child, &child_id, plain, html)?;
    }

    Ok(())
}

fn decode_gmail_body(data: &str, part_id: &str) -> anyhow::Result<String> {
    let mut normalized = data.trim().to_owned();
    while !normalized.len().is_multiple_of(4) {
        normalized.push('=');
    }

    let bytes =
        URL_SAFE
            .decode(normalized.as_bytes())
            .map_err(|source| {
                anyhow::anyhow!(
                    "failed to decode Gmail message body for part `{part_id}`: {source}"
                )
            })?;
    String::from_utf8(bytes).map_err(|source| {
        anyhow::anyhow!("Gmail message body for part `{part_id}` is not valid UTF-8: {source}")
    })
}

fn html_to_searchable_text(content: &str) -> String {
    let without_tags = strip_html_tags(content);
    decode_basic_entities(&without_tags)
}

fn strip_html_tags(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut inside_tag = false;

    for ch in content.chars() {
        match ch {
            '<' => {
                inside_tag = true;
                output.push(' ');
            }
            '>' => {
                inside_tag = false;
                output.push(' ');
            }
            _ if !inside_tag => output.push(ch),
            _ => {}
        }
    }

    output
}

fn decode_basic_entities(content: &str) -> String {
    content
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x2F;", "/")
        .replace("&#47;", "/")
}

fn normalize_code(raw: &str) -> String {
    raw.chars().filter(char::is_ascii_digit).collect()
}

fn looks_like_false_positive(code: &str) -> bool {
    matches!(code, "177010") || code.chars().all(|ch| ch == '0')
}

#[cfg(test)]
mod tests {
    use super::{
        ExtractCodeOptions, collect_message_body_candidates, extract_verification_code,
        extract_verification_code_with_options,
    };
    use crate::model::{GmailMessage, GmailMessagePart};
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    #[test]
    fn extracts_explicit_english_code() {
        assert_eq!(
            extract_verification_code("Your verification code: 123456").as_deref(),
            Some("123456")
        );
    }

    #[test]
    fn extracts_chinese_verification_code() {
        assert_eq!(
            extract_verification_code("验证码： 876 543").as_deref(),
            Some("876543")
        );
    }

    #[test]
    fn ignores_hex_color_before_real_code() {
        let content = "style=\"color:#123456\" code is 778899";

        assert_eq!(
            extract_verification_code(content).as_deref(),
            Some("778899")
        );
    }

    #[test]
    fn supports_custom_digit_length() {
        let options = ExtractCodeOptions::new(9, 10);

        assert_eq!(
            extract_verification_code_with_options("OTP: 1234567890", options).as_deref(),
            Some("1234567890")
        );
    }

    #[test]
    fn body_candidates_prefer_plain_text_over_html() {
        let message = GmailMessage {
            id: "m1".to_owned(),
            thread_id: None,
            snippet: None,
            payload: Some(GmailMessagePart {
                part_id: Some("root".to_owned()),
                mime_type: "multipart/alternative".to_owned(),
                headers: Vec::new(),
                body: None,
                parts: vec![
                    GmailMessagePart {
                        part_id: Some("1".to_owned()),
                        mime_type: "text/html".to_owned(),
                        headers: Vec::new(),
                        body: Some(crate::model::GmailMessagePartBody {
                            data: Some(encode("<b>999999</b>")),
                            size: None,
                            attachment_id: None,
                        }),
                        parts: Vec::new(),
                    },
                    GmailMessagePart {
                        part_id: Some("2".to_owned()),
                        mime_type: "text/plain".to_owned(),
                        headers: Vec::new(),
                        body: Some(crate::model::GmailMessagePartBody {
                            data: Some(encode("Your code is 123456")),
                            size: None,
                            attachment_id: None,
                        }),
                        parts: Vec::new(),
                    },
                ],
            }),
            extra: Default::default(),
        };

        let candidates = collect_message_body_candidates(&message).expect("candidates");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].mime_type, "text/plain");
        assert!(candidates[0].text.contains("123456"));
    }

    fn encode(value: &str) -> String {
        URL_SAFE_NO_PAD.encode(value.as_bytes())
    }
}
