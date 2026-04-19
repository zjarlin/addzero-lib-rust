#![forbid(unsafe_code)]

use deunicode::deunicode;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableType {
    Constant,
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
}

pub trait ParentPathExt {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatArg {
    Null,
    String(String),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    Boolean(bool),
}

impl FormatArg {
    fn as_text(&self) -> String {
        match self {
            Self::Null => "null".to_owned(),
            Self::String(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Unsigned(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
        }
    }

    fn as_f64(&self) -> f64 {
        match self {
            Self::Float(value) => *value,
            Self::Integer(value) => *value as f64,
            Self::Unsigned(value) => *value as f64,
            Self::Boolean(value) => usize::from(*value) as f64,
            Self::String(value) => value.parse::<f64>().unwrap_or(0.0),
            Self::Null => 0.0,
        }
    }

    fn as_i64(&self) -> i64 {
        match self {
            Self::Integer(value) => *value,
            Self::Unsigned(value) => *value as i64,
            Self::Float(value) => *value as i64,
            Self::Boolean(value) => i64::from(*value),
            Self::String(value) => value.parse::<i64>().unwrap_or(0),
            Self::Null => 0,
        }
    }

    fn as_u64(&self) -> u64 {
        match self {
            Self::Unsigned(value) => *value,
            Self::Integer(value) => (*value).max(0) as u64,
            Self::Float(value) => value.max(0.0) as u64,
            Self::Boolean(value) => u64::from(*value),
            Self::String(value) => value.parse::<u64>().unwrap_or(0),
            Self::Null => 0,
        }
    }
}

impl From<&str> for FormatArg {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for FormatArg {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for FormatArg {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

macro_rules! impl_from_signed {
    ($($ty:ty),* $(,)?) => {
        $(impl From<$ty> for FormatArg {
            fn from(value: $ty) -> Self {
                Self::Integer(value as i64)
            }
        })*
    };
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(impl From<$ty> for FormatArg {
            fn from(value: $ty) -> Self {
                Self::Unsigned(value as u64)
            }
        })*
    };
}

macro_rules! impl_from_float {
    ($($ty:ty),* $(,)?) => {
        $(impl From<$ty> for FormatArg {
            fn from(value: $ty) -> Self {
                Self::Float(value as f64)
            }
        })*
    };
}

impl_from_signed!(i8, i16, i32, i64, isize);
impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_float!(f32, f64);

pub fn clean_blank(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    if input.is_empty() {
        return String::new();
    }

    whitespace_regex()
        .replace_all(input.trim(), " ")
        .chars()
        .filter(|character| is_visible(*character))
        .collect()
}

pub fn default_table_english_name(
    table_english_name: impl AsRef<str>,
    table_chinese_name: Option<&str>,
) -> String {
    let table_english_name = table_english_name.as_ref();
    let seed = if table_english_name.trim().is_empty() {
        table_chinese_name.unwrap_or_default()
    } else {
        table_english_name
    };

    let without_parenthetical = parenthetical_regex().replace_all(seed, "");
    let transliterated = if table_english_name.trim().is_empty() {
        deunicode(without_parenthetical.trim())
    } else {
        without_parenthetical.to_string()
    };

    let sanitized: String = transliterated
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();

    underscore_regex()
        .replace_all(sanitized.trim_matches('_'), "_")
        .to_string()
}

pub fn parent_path_and_mkdir(
    path: impl AsRef<Path>,
    child_path: impl AsRef<Path>,
) -> io::Result<PathBuf> {
    let path = path.as_ref();
    let Some(parent) = path.parent() else {
        let target = PathBuf::from(child_path.as_ref());
        fs::create_dir_all(&target)?;
        return Ok(target);
    };

    let target = parent.join(child_path.as_ref());
    fs::create_dir_all(&target)?;
    Ok(target)
}

impl ParentPathExt for Path {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        parent_path_and_mkdir(self, child_path)
    }
}

impl ParentPathExt for str {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        parent_path_and_mkdir(self, child_path)
    }
}

pub fn group_by_separator<T, F>(lines: &[T], predicate: F) -> HashMap<T, Vec<T>>
where
    T: Clone + Eq + Hash,
    F: Fn(&T) -> bool,
{
    let separator_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, item)| predicate(item).then_some(index))
        .collect::<Vec<_>>();

    let mut result = HashMap::with_capacity(separator_indexes.len());
    for (position, separator_index) in separator_indexes.iter().enumerate() {
        let next = separator_indexes
            .get(position + 1)
            .copied()
            .unwrap_or(lines.len());
        result.insert(
            lines[*separator_index].clone(),
            lines[*separator_index + 1..next].to_vec(),
        );
    }
    result
}

pub fn make_surround_with(input: Option<&str>, fix: &str) -> String {
    let with_prefix = add_prefix_if_not(input, fix, false);
    add_suffix_if_not(Some(&with_prefix), fix, false)
}

pub fn make_surround_with_html_p(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    if input.trim().is_empty() {
        return String::new();
    }

    let with_prefix = add_prefix_if_not(Some(input), "<p>", false);
    add_suffix_if_not(Some(&with_prefix), "</p>", false)
}

pub fn make_surround_with_brackets(input: &str) -> String {
    format!("({input})")
}

pub fn remove_not_chinese(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    input
        .chars()
        .filter(|character| is_chinese(*character))
        .collect()
}

pub fn add_suffix_if_not(input: Option<&str>, suffix: &str, ignore_case: bool) -> String {
    let Some(input) = input else {
        return suffix.to_owned();
    };
    if ends_with_ignore_case(input, suffix, ignore_case) {
        input.to_owned()
    } else {
        format!("{input}{suffix}")
    }
}

pub fn add_prefix_if_not(input: Option<&str>, prefix: &str, ignore_case: bool) -> String {
    let Some(input) = input else {
        return String::new();
    };
    if input.is_empty() {
        return String::new();
    }
    if starts_with_ignore_case(input, prefix, ignore_case) {
        input.to_owned()
    } else {
        format!("{prefix}{input}")
    }
}

pub fn is_not_blank(input: Option<&str>) -> bool {
    !is_blank(input)
}

pub fn get_path_from_right(input: &str, n: usize) -> String {
    let parts = input
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < n {
        return input.to_owned();
    }
    parts[..parts.len() - n].join(".")
}

pub fn lower_case(input: Option<&str>) -> String {
    input.unwrap_or_default().to_lowercase()
}

pub fn lower_first(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut result = first.to_lowercase().collect::<String>();
    result.push_str(chars.as_str());
    result
}

pub fn ignore_case_in<I, S>(value: &str, collection: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    collection
        .into_iter()
        .any(|item| item.as_ref().eq_ignore_ascii_case(value))
}

pub fn ignore_case_not_in<I, S>(value: &str, collection: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    !ignore_case_in(value, collection)
}

pub fn contains_any_ignore_case<I, S>(value: &str, substrings: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let lower_value = value.to_lowercase();
    substrings.into_iter().any(|substring| {
        let substring = substring.as_ref();
        !substring.is_empty() && lower_value.contains(&substring.to_lowercase())
    })
}

pub fn ignore_case_like(value: &str, other: &str) -> bool {
    value.to_lowercase().contains(&other.to_lowercase())
}

pub fn contains_chinese(input: Option<&str>) -> bool {
    input.unwrap_or_default().chars().any(is_chinese)
}

pub fn join<S: AsRef<str>>(separator: &str, values: &[S]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(separator)
}

pub fn with_pkg(base: &str, pkg: &str) -> String {
    format!("{base}/{}", pkg.replace('.', "/"))
}

pub fn with_file_name(base: &str, file_name: &str) -> String {
    format!("{base}/{file_name}")
}

pub fn with_file_suffix(base: &str, suffix: Option<&str>) -> String {
    format!("{base}{}", suffix.unwrap_or(".kt"))
}

pub fn remove_any<I, S>(input: Option<&str>, strings_to_remove: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let Some(input) = input else {
        return String::new();
    };
    strings_to_remove
        .into_iter()
        .fold(input.to_owned(), |current, value| {
            current.replace(value.as_ref(), "")
        })
}

pub fn remove_any_quote(input: &str) -> String {
    remove_any(Some(input), ["\"", "\\"])
}

pub fn remove_blank_or_quotation(input: &str) -> String {
    remove_any(Some(input), [" ", "\""])
}

pub fn to_underline_case(input: &str) -> String {
    join_identifier_words(input, "_", CaseStyle::Preserve)
}

pub fn to_underline_lower_case(input: &str) -> String {
    to_underline_case(input).to_lowercase()
}

pub fn is_number(input: &str) -> bool {
    number_regex().is_match(input)
}

pub fn equals_ignore_case(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

pub fn to_not_empty_str<T: ToString>(value: Option<T>) -> String {
    value
        .map(|value| remove_blank_or_quotation(&value.to_string()))
        .unwrap_or_default()
}

pub fn kmp_format(template: &str, args: &[FormatArg]) -> String {
    format_template(template, args)
}

pub fn format_decimal(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

pub fn format_decimal_f32(value: f32, decimals: usize) -> String {
    format_decimal(value as f64, decimals)
}

pub fn format_currency(value: f64, decimals: usize) -> String {
    format_decimal(value, decimals)
}

pub fn format_currency_f32(value: f32, decimals: usize) -> String {
    format_currency(value as f64, decimals)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmpMatcher {
    pattern: String,
    lps: Vec<usize>,
}

impl KmpMatcher {
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let lps = compute_lps(&pattern);
        Self { pattern, lps }
    }

    pub fn search(&self, text: &str) -> isize {
        if self.pattern.is_empty() {
            return 0;
        }

        let pattern = self.pattern.as_bytes();
        let text = text.as_bytes();
        let mut text_index = 0usize;
        let mut pattern_index = 0usize;

        while text_index < text.len() {
            if pattern[pattern_index] == text[text_index] {
                text_index += 1;
                pattern_index += 1;
            }

            if pattern_index == pattern.len() {
                return (text_index - pattern_index) as isize;
            }

            if text_index < text.len() && pattern[pattern_index] != text[text_index] {
                if pattern_index != 0 {
                    pattern_index = self.lps[pattern_index - 1];
                } else {
                    text_index += 1;
                }
            }
        }

        -1
    }

    pub fn search_all(&self, text: &str) -> Vec<usize> {
        if self.pattern.is_empty() {
            return Vec::new();
        }

        let pattern = self.pattern.as_bytes();
        let text = text.as_bytes();
        let mut matches = Vec::new();
        let mut text_index = 0usize;
        let mut pattern_index = 0usize;

        while text_index < text.len() {
            if pattern[pattern_index] == text[text_index] {
                text_index += 1;
                pattern_index += 1;
            }

            if pattern_index == pattern.len() {
                matches.push(text_index - pattern_index);
                pattern_index = self.lps[pattern_index - 1];
            } else if text_index < text.len() && pattern[pattern_index] != text[text_index] {
                if pattern_index != 0 {
                    pattern_index = self.lps[pattern_index - 1];
                } else {
                    text_index += 1;
                }
            }
        }

        matches
    }
}

pub fn contains_kmp(text: &str, pattern: &str) -> bool {
    KmpMatcher::new(pattern).search(text) != -1
}

pub fn index_of_kmp(text: &str, pattern: &str) -> isize {
    KmpMatcher::new(pattern).search(text)
}

pub fn find_all_kmp(text: &str, pattern: &str) -> Vec<usize> {
    KmpMatcher::new(pattern).search_all(text)
}

pub fn replace_kmp(text: &str, pattern: &str, replacement: &str) -> String {
    if pattern.is_empty() {
        return text.to_owned();
    }

    let indices = find_all_kmp(text, pattern);
    if indices.is_empty() {
        return text.to_owned();
    }

    let mut result = String::new();
    let mut last_index = 0usize;
    for index in indices {
        if index < last_index {
            continue;
        }
        result.push_str(&text[last_index..index]);
        result.push_str(replacement);
        last_index = index + pattern.len();
    }
    result.push_str(&text[last_index..]);
    result
}

pub fn remove_last_char_occurrence(input: Option<&str>, target: char) -> String {
    let Some(input) = input else {
        return String::new();
    };
    if input.trim().is_empty() {
        return String::new();
    }
    let Some(index) = input.rfind(target) else {
        return input.to_owned();
    };
    let mut result = String::with_capacity(input.len().saturating_sub(target.len_utf8()));
    result.push_str(&input[..index]);
    result.push_str(&input[index + target.len_utf8()..]);
    result
}

pub fn extract_markdown_block_content(markdown: Option<&str>) -> String {
    let Some(markdown) = markdown else {
        return String::new();
    };
    if markdown.is_empty() {
        return String::new();
    }

    if markdown.contains("```") || markdown.contains("json") {
        return fenced_block_regex()
            .captures(markdown)
            .and_then(|captures| captures.get(1))
            .map(|matched| matched.as_str().trim().to_owned())
            .unwrap_or_default();
    }

    markdown.to_owned()
}

pub fn extract_code_block_content(code: impl AsRef<str>) -> String {
    double_tick_regex()
        .captures(code.as_ref())
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str().trim().to_owned())
        .unwrap_or_default()
}

pub fn to_valid_variable_name(
    input: &str,
    variable_type: VariableType,
    prefix: &str,
    suffix: &str,
) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    if input.chars().all(|character| character.is_ascii_digit()) {
        return format!("__{input}");
    }

    let mut cleaned = invalid_identifier_regex()
        .replace_all(input, "")
        .to_string();
    if cleaned.trim().is_empty() {
        return input.to_owned();
    }
    if cleaned
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        cleaned.insert(0, '_');
    }

    let words = split_words(&cleaned)
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>();
    if words.is_empty() {
        return cleaned;
    }

    let mut result = match variable_type {
        VariableType::Constant => words
            .iter()
            .map(|word| word.to_uppercase())
            .collect::<Vec<_>>()
            .join("_"),
        VariableType::CamelCase => {
            let mut result = words[0].clone();
            for word in &words[1..] {
                result.push_str(&capitalize(word));
            }
            result
        }
        VariableType::PascalCase => words.iter().map(|word| capitalize(word)).collect(),
        VariableType::SnakeCase => words.join("_"),
        VariableType::KebabCase => words.join("-"),
    };

    if !prefix.trim().is_empty() {
        result = match variable_type {
            VariableType::Constant => format!("{}_{}", prefix.to_uppercase(), result),
            VariableType::CamelCase => format!("{}{}", prefix.to_lowercase(), capitalize(&result)),
            VariableType::PascalCase => format!("{}{}", capitalize(prefix), result),
            VariableType::SnakeCase => format!("{}_{}", prefix.to_lowercase(), result),
            VariableType::KebabCase => format!("{}-{}", prefix.to_lowercase(), result),
        };
    }

    if !suffix.trim().is_empty() {
        result = match variable_type {
            VariableType::Constant => format!("{}_{}", result, suffix.to_uppercase()),
            VariableType::CamelCase | VariableType::PascalCase => {
                format!("{}{}", result, capitalize(suffix))
            }
            VariableType::SnakeCase => format!("{}_{}", result, suffix.to_lowercase()),
            VariableType::KebabCase => format!("{}-{}", result, suffix.to_lowercase()),
        };
    }

    result
}

pub fn to_constant_name(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::Constant, prefix, suffix)
}

pub fn to_camel_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::CamelCase, prefix, suffix)
}

pub fn to_pascal_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::PascalCase, prefix, suffix)
}

pub fn to_snake_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::SnakeCase, prefix, suffix)
}

pub fn to_kebab_name(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::KebabCase, prefix, suffix)
}

pub fn length(input: Option<&str>) -> usize {
    input.map_or(0, str::len)
}

pub fn contains_any<I, S>(input: Option<&str>, test_strings: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let Some(input) = input else {
        return false;
    };
    test_strings
        .into_iter()
        .any(|test| input.contains(test.as_ref()))
}

pub fn is_blank(input: Option<&str>) -> bool {
    input.is_none_or(|value| value.trim().is_empty())
}

pub fn is_null_or_empty(input: Option<&str>) -> bool {
    input.is_none_or(str::is_empty)
}

pub fn remove_duplicate_symbol(
    source: Option<&str>,
    duplicate_element: Option<&str>,
) -> Option<String> {
    let source = source?;
    let duplicate_element = duplicate_element?;
    if duplicate_element.is_empty() {
        return Some(source.to_owned());
    }

    let pattern = format!("(?:{})+", regex::escape(duplicate_element));
    let regex = Regex::new(&pattern).expect("regex must compile");
    Some(regex.replace_all(source, duplicate_element).to_string())
}

pub fn extract_text_between_p_tags(input: Option<&str>) -> Vec<String> {
    let Some(input) = input else {
        return Vec::new();
    };
    if !input.contains("<p>") || !input.contains("</p>") {
        return Vec::new();
    }

    p_tag_regex()
        .captures_iter(input)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_owned()))
        .collect()
}

pub fn clean_doc_comment(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    whitespace_regex()
        .replace_all(
            &doc_comment_marker_regex()
                .replace_all(input, " ")
                .replace('\n', " "),
            " ",
        )
        .trim()
        .to_owned()
}

pub fn first_not_blank(values: &[Option<&str>]) -> String {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
        .unwrap_or_default()
}

pub fn get_rest_url(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    rest_url_regex()
        .captures(input)
        .and_then(|captures| captures.get(2))
        .map(|value| value.as_str().to_owned())
        .unwrap_or_default()
}

pub fn extract_key_value_pairs(input: &str) -> HashMap<String, String> {
    key_value_regex()
        .captures_iter(input)
        .filter_map(|captures| {
            let key = captures.get(1)?.as_str().trim().to_owned();
            let value = captures.get(2)?.as_str().trim().to_owned();
            Some((key, value))
        })
        .collect()
}

pub fn escape_special_characters(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\'' => output.push_str("\\'"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '&' => output.push_str("&amp;"),
            ';' => output.push_str("\\;"),
            '`' => output.push_str("\\`"),
            '$' => output.push_str("\\$"),
            '!' => output.push_str("\\!"),
            '%' => output.push_str("\\%"),
            '#' => output.push_str("\\#"),
            '~' => output.push_str("\\~"),
            '=' => output.push_str("\\="),
            '+' => output.push_str("\\+"),
            '[' | ']' | '{' | '}' | '(' | ')' | '^' | '.' | '|' | '*' | '?' => {
                output.push('\\');
                output.push(character);
            }
            _ if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            _ => output.push(character),
        }
    }

    output
        .replace("--", "\\--")
        .replace("/*", "\\/\\*")
        .replace("*/", "*\\/")
}

pub fn to_kebab_case(input: &str) -> String {
    join_identifier_words(input, "-", CaseStyle::Lower)
}

pub fn format_template(template: &str, args: &[FormatArg]) -> String {
    let mut result = String::new();
    let mut arg_index = 0usize;
    let mut index = 0usize;
    let bytes = template.as_bytes();

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 1 < bytes.len() {
            match bytes[index + 1] as char {
                '%' => {
                    result.push('%');
                    index += 2;
                }
                's' | 'S' => {
                    result.push_str(&args.get(arg_index).unwrap_or(&FormatArg::Null).as_text());
                    arg_index += 1;
                    index += 2;
                }
                'd' => {
                    result.push_str(
                        &args
                            .get(arg_index)
                            .unwrap_or(&FormatArg::Null)
                            .as_i64()
                            .to_string(),
                    );
                    arg_index += 1;
                    index += 2;
                }
                'f' => {
                    let value = args.get(arg_index).unwrap_or(&FormatArg::Null).as_f64();
                    result.push_str(&value.to_string());
                    arg_index += 1;
                    index += 2;
                }
                'x' => {
                    result.push_str(&format!(
                        "{:x}",
                        args.get(arg_index).unwrap_or(&FormatArg::Null).as_u64()
                    ));
                    arg_index += 1;
                    index += 2;
                }
                '.' => {
                    let mut precision_end = index + 2;
                    while precision_end < bytes.len() && bytes[precision_end].is_ascii_digit() {
                        precision_end += 1;
                    }
                    if precision_end < bytes.len() && bytes[precision_end] == b'f' {
                        let precision = template[index + 2..precision_end]
                            .parse::<usize>()
                            .unwrap_or(2);
                        let value = args.get(arg_index).unwrap_or(&FormatArg::Null).as_f64();
                        result.push_str(&format!("{value:.precision$}"));
                        arg_index += 1;
                        index = precision_end + 1;
                    } else {
                        result.push('%');
                        index += 1;
                    }
                }
                _ => {
                    result.push('%');
                    index += 1;
                }
            }
            continue;
        }

        result.push(bytes[index] as char);
        index += 1;
    }

    result
}

pub fn to_simple_name(input: &str) -> String {
    input.rsplit('.').next().unwrap_or_default().to_owned()
}

fn compute_lps(pattern: &str) -> Vec<usize> {
    let bytes = pattern.as_bytes();
    let mut lps = vec![0; bytes.len()];
    let mut len = 0usize;
    let mut index = 1usize;

    while index < bytes.len() {
        if bytes[index] == bytes[len] {
            len += 1;
            lps[index] = len;
            index += 1;
        } else if len != 0 {
            len = lps[len - 1];
        } else {
            lps[index] = 0;
            index += 1;
        }
    }

    lps
}

fn split_words(input: &str) -> Vec<String> {
    let normalized = input
        .chars()
        .map(|character| match character {
            '-' | '_' => ' ',
            _ => character,
        })
        .collect::<String>();

    let mut words = Vec::new();
    for token in normalized.split_whitespace() {
        words.extend(split_token(token));
    }
    words
}

fn split_token(token: &str) -> Vec<String> {
    let chars = token.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut words = Vec::new();
    let mut current = String::new();

    for index in 0..chars.len() {
        let current_char = chars[index];
        let previous = (index > 0).then(|| chars[index - 1]);
        let next = chars.get(index + 1).copied();

        let boundary = previous.is_some_and(|previous| {
            (current_char.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit()))
                || (current_char.is_ascii_uppercase()
                    && previous.is_ascii_uppercase()
                    && next.is_some_and(|next| next.is_ascii_lowercase()))
        });

        if boundary && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        current.push(current_char);
    }

    if !current.is_empty() {
        words.push(current);
    }
    words
}

enum CaseStyle {
    Preserve,
    Lower,
}

fn join_identifier_words(input: &str, separator: &str, case_style: CaseStyle) -> String {
    split_words(input)
        .into_iter()
        .map(|word| match case_style {
            CaseStyle::Preserve => word,
            CaseStyle::Lower => word.to_lowercase(),
        })
        .collect::<Vec<_>>()
        .join(separator)
}

fn capitalize(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut output = first.to_uppercase().collect::<String>();
    output.push_str(chars.as_str());
    output
}

fn starts_with_ignore_case(input: &str, prefix: &str, ignore_case: bool) -> bool {
    if ignore_case {
        input.to_lowercase().starts_with(&prefix.to_lowercase())
    } else {
        input.starts_with(prefix)
    }
}

fn ends_with_ignore_case(input: &str, suffix: &str, ignore_case: bool) -> bool {
    if ignore_case {
        input.to_lowercase().ends_with(&suffix.to_lowercase())
    } else {
        input.ends_with(suffix)
    }
}

fn is_visible(character: char) -> bool {
    matches!(character as u32, 32..=126 | 0x4E00..=0x9FFF)
}

fn is_chinese(character: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&character)
}

fn whitespace_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s+").expect("regex must compile"))
}

fn parenthetical_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\((.*?)\)").expect("regex must compile"))
}

fn underscore_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"_{2,}").expect("regex must compile"))
}

fn number_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^-?\d*\.?\d+$").expect("regex must compile"))
}

fn invalid_identifier_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[^a-zA-Z0-9\s_-]").expect("regex must compile"))
}

fn p_tag_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)<p>(.*?)</p>").expect("regex must compile"))
}

fn fenced_block_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)```[\w-]*\s*(.*?)\s*```").expect("regex must compile"))
}

fn double_tick_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)``\w*\s*(.*?)\s*``").expect("regex must compile"))
}

fn doc_comment_marker_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"/\*\*?|\*/|\*|/").expect("regex must compile"))
}

fn rest_url_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r".*:\d+(/[^/]+)(/.*)").expect("regex must compile"))
}

fn key_value_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"([\p{L}\p{N}_]+)[ \t]*[:：][ \t]*([\p{L}\p{N}_]+)")
            .expect("regex must compile")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn clean_blank_normalizes_whitespace_and_invisible_chars() {
        let value = clean_blank(Some("  hello\t\t世界\u{0001}\n "));
        assert_eq!(value, "hello 世界");
    }

    #[test]
    fn default_table_name_prefers_existing_name_and_sanitizes_it() {
        let result = default_table_english_name("user_profile(test)", Some("用户信息"));
        assert_eq!(result, "user_profile");
    }

    #[test]
    fn default_table_name_transliterates_when_english_name_is_blank() {
        let result = default_table_english_name("", Some("用户(表)"));
        assert!(!result.is_empty());
        assert!(!result.contains('('));
        assert!(
            result
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        );
    }

    #[test]
    fn parent_path_and_mkdir_creates_child_directory() {
        let temp = TempDir::new().expect("temp dir should exist");
        let file_path = temp.path().join("logs/app.log");

        let created = parent_path_and_mkdir(&file_path, "archive").expect("directory should exist");

        assert_eq!(created, temp.path().join("logs/archive"));
        assert!(created.is_dir());
    }

    #[test]
    fn trait_extension_for_parent_path_works_on_str() {
        let temp = TempDir::new().expect("temp dir should exist");
        let file_path = temp.path().join("reports/export.txt");
        let created = file_path
            .to_string_lossy()
            .as_ref()
            .parent_path_and_mkdir("history")
            .expect("directory should be created");

        assert_eq!(created, temp.path().join("reports/history"));
    }

    #[test]
    fn naming_conversions_cover_camel_pascal_snake_and_kebab() {
        assert_eq!(to_camel_case("sys_yes_no", "", ""), "sysYesNo");
        assert_eq!(to_pascal_case("propSource", "", ""), "PropSource");
        assert_eq!(to_snake_case("XMLHttpRequest", "", ""), "xml_http_request");
        assert_eq!(
            to_kebab_name("hello_world-test", "", ""),
            "hello-world-test"
        );
        assert_eq!(to_constant_name("max value", "", ""), "MAX_VALUE");
    }

    #[test]
    fn to_valid_variable_name_handles_digits_prefix_and_suffix() {
        assert_eq!(
            to_valid_variable_name("123", VariableType::CamelCase, "", ""),
            "__123"
        );
        assert_eq!(
            to_valid_variable_name("user name", VariableType::CamelCase, "my", "dto"),
            "myUserNameDto"
        );
        assert_eq!(
            to_valid_variable_name("order item", VariableType::SnakeCase, "erp", "entity"),
            "erp_order_item_entity"
        );
    }

    #[test]
    fn underline_and_kebab_case_use_identifier_boundaries() {
        assert_eq!(to_underline_case("userName"), "user_Name");
        assert_eq!(
            to_underline_lower_case("XMLHttpRequest"),
            "xml_http_request"
        );
        assert_eq!(
            to_kebab_case("hello_world TestCase"),
            "hello-world-test-case"
        );
    }

    #[test]
    fn prefix_suffix_and_remove_helpers_work() {
        assert_eq!(
            add_prefix_if_not(Some("world"), "hello ", false),
            "hello world"
        );
        assert_eq!(add_suffix_if_not(Some("file"), ".txt", false), "file.txt");
        assert_eq!(remove_any(Some(r#" "ab\c" "#), [" ", "\"", "\\"]), "abc");
        assert_eq!(
            remove_duplicate_symbol(Some("a----b"), Some("-")),
            Some("a-b".to_owned())
        );
    }

    #[test]
    fn kmp_search_and_replace_work() {
        let matcher = KmpMatcher::new("aba");
        assert_eq!(matcher.search("xxabaxx"), 2);
        assert_eq!(matcher.search_all("ababa"), vec![0, 2]);
        assert!(contains_kmp("hello world", "world"));
        assert_eq!(replace_kmp("ababa", "aba", "X"), "Xba");
    }

    #[test]
    fn extraction_helpers_work() {
        let pairs = extract_key_value_pairs("姓名：张三 年龄：25 city:Beijing");
        assert_eq!(pairs.get("姓名"), Some(&"张三".to_owned()));
        assert_eq!(pairs.get("年龄"), Some(&"25".to_owned()));
        assert_eq!(pairs.get("city"), Some(&"Beijing".to_owned()));

        assert_eq!(
            extract_text_between_p_tags(Some("<p>a</p><div>x</div><p>b</p>")),
            vec!["a".to_owned(), "b".to_owned()]
        );
        assert_eq!(
            get_rest_url(Some("http://localhost:8080/api/users/list")),
            "/users/list"
        );
    }

    #[test]
    fn markdown_and_code_block_extractors_work() {
        let markdown = "before\n```json\n{\"name\":\"addzero\"}\n```\nafter";
        assert_eq!(
            extract_markdown_block_content(Some(markdown)),
            "{\"name\":\"addzero\"}"
        );
        assert_eq!(
            extract_markdown_block_content(Some("plain text")),
            "plain text"
        );
        assert_eq!(
            extract_code_block_content("``sql\nselect * from users\n``"),
            "select * from users"
        );
    }

    #[test]
    fn doc_comment_and_blank_helpers_work() {
        let cleaned = clean_doc_comment(Some("/**\n * hello world\n */"));
        assert_eq!(cleaned, "hello world");
        assert_eq!(first_not_blank(&[None, Some(""), Some("  ok  ")]), "  ok  ");
        assert!(contains_chinese(Some("hello世界")));
        assert!(contains_any_ignore_case("HelloWorld", ["world", "test"]));
    }

    #[test]
    fn format_helpers_work() {
        let value = format_template(
            "Name: %s, Age: %d, Score: %.1f, Hex: %x, Done: %%",
            &["John".into(), 30.into(), 95.5.into(), 255.into()],
        );
        assert_eq!(value, "Name: John, Age: 30, Score: 95.5, Hex: ff, Done: %");
        assert_eq!(kmp_format("Value: %.2f", &[3.14159.into()]), "Value: 3.14");
        assert_eq!(format_currency(19.99, 2), "19.99");
    }

    #[test]
    fn text_and_misc_helpers_work() {
        assert_eq!(remove_not_chinese(Some("abc中文123")), "中文");
        assert_eq!(get_path_from_right("a.b.c", 1), "a.b");
        assert_eq!(lower_first("UserName"), "userName");
        assert_eq!(to_simple_name("site.addzero.UserName"), "UserName");
        assert_eq!(escape_special_characters("<a&b>"), "&lt;a&amp;b&gt;");
        assert!(is_number("-12.5"));
    }
}
