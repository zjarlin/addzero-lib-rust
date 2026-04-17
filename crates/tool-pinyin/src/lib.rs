use pinyin::{Pinyin, ToPinyin, ToPinyinMulti};
use regex::Regex;
use std::sync::OnceLock;

pub fn sanitize(input: impl AsRef<str>, default_name: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let default_name = default_name.as_ref();
    if input.trim().is_empty() {
        return default_name.to_uppercase();
    }

    let sanitized = invalid_name_regex().replace_all(input, "_");
    let sanitized = duplicate_underscore_regex().replace_all(&sanitized, "_");
    let sanitized = sanitized.trim_matches('_');

    if sanitized.is_empty() {
        return default_name.to_uppercase();
    }

    let prefixed = if sanitized
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        format!("__{sanitized}")
    } else {
        sanitized.to_owned()
    };

    prefixed.to_uppercase()
}

pub fn string_to_pinyin(
    src: Option<&str>,
    is_polyphone: bool,
    separator: Option<&str>,
) -> Option<Vec<String>> {
    src.map(|value| {
        value
            .chars()
            .map(|character| char_to_pinyin(character, is_polyphone, separator))
            .collect()
    })
}

pub fn char_to_pinyin(src: char, is_polyphone: bool, separator: Option<&str>) -> String {
    if is_polyphone {
        if let Some(multi) = src.to_pinyin_multi() {
            let separator = separator.unwrap_or("");
            return multi
                .into_iter()
                .map(Pinyin::plain)
                .collect::<Vec<_>>()
                .join(separator);
        }
    } else if let Some(pinyin) = src.to_pinyin() {
        return pinyin.plain().to_owned();
    }

    src.to_string()
}

pub fn hanzi_to_pinyin(hanzi: impl AsRef<str>, separator: Option<&str>) -> String {
    let separator = separator.unwrap_or(" ");
    hanzi
        .as_ref()
        .chars()
        .map(|character| char_to_pinyin(character, false, None))
        .collect::<Vec<_>>()
        .join(separator)
}

pub fn string_array_to_string(parts: &[impl AsRef<str>], separator: Option<&str>) -> String {
    parts
        .iter()
        .map(|part| part.as_ref())
        .collect::<Vec<_>>()
        .join(separator.unwrap_or(""))
}

pub fn char_array_to_string(parts: &[char], separator: Option<&str>) -> String {
    parts
        .iter()
        .map(char::to_string)
        .collect::<Vec<_>>()
        .join(separator.unwrap_or(" "))
}

pub fn get_head_by_char(src: char, is_capital: bool) -> Vec<char> {
    if let Some(multi) = src.to_pinyin_multi() {
        multi
            .into_iter()
            .map(Pinyin::first_letter)
            .filter_map(|value| value.chars().next())
            .map(|character| {
                if is_capital {
                    character.to_ascii_uppercase()
                } else {
                    character
                }
            })
            .collect()
    } else {
        vec![if is_capital {
            src.to_ascii_uppercase()
        } else {
            src
        }]
    }
}

pub fn get_head_by_string(
    src: impl AsRef<str>,
    is_capital: bool,
    separator: Option<&str>,
) -> Vec<String> {
    src.as_ref()
        .chars()
        .map(|character| {
            let heads = get_head_by_char(character, is_capital);
            char_array_to_string(&heads, separator)
        })
        .collect()
}

fn invalid_name_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[^a-zA-Z0-9]").expect("regex must compile"))
}

fn duplicate_underscore_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"_+").expect("regex must compile"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_creates_uppercase_identifier() {
        assert_eq!(sanitize("hello-world", "code"), "HELLO_WORLD");
        assert_eq!(sanitize("123", "code"), "__123");
        assert_eq!(sanitize("", "code"), "CODE");
    }

    #[test]
    fn string_and_char_to_pinyin_convert_hanzi() {
        let chars = string_to_pinyin(Some("中国"), false, Some(",")).expect("should convert");

        assert_eq!(chars, vec!["zhong".to_owned(), "guo".to_owned()]);
        assert_eq!(char_to_pinyin('中', false, None), "zhong");
    }

    #[test]
    fn polyphone_conversion_joins_all_readings() {
        let value = char_to_pinyin('还', true, Some(","));

        assert!(value.contains("hai"));
        assert!(value.contains(','));
    }

    #[test]
    fn hanzi_to_pinyin_and_join_helpers_work() {
        assert_eq!(hanzi_to_pinyin("中国", Some("_")), "zhong_guo");
        assert_eq!(string_array_to_string(&["a", "b", "c"], Some("-")), "a-b-c");
        assert_eq!(char_array_to_string(&['A', 'B'], Some("")), "AB");
    }

    #[test]
    fn head_helpers_return_initials() {
        let heads = get_head_by_char('中', true);
        assert!(!heads.is_empty());
        assert!(heads.iter().all(|head| *head == 'Z'));

        let words = get_head_by_string("中国", true, Some(""));
        assert_eq!(words[1], "G");
        assert!(words[0].chars().all(|head| head == 'Z'));
    }
}
