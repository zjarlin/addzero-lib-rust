use deunicode::deunicode;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// 标识符输出风格，用于把任意文本规整成代码生成可用的变量名。
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    derive_more::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub enum VariableType {
    /// 常量风格，例如 `MAX_VALUE`。
    Constant,
    /// 小驼峰风格，例如 `maxValue`。
    CamelCase,
    /// 大驼峰风格，例如 `MaxValue`。
    PascalCase,
    /// 下划线风格，例如 `max_value`。
    SnakeCase,
    /// 短横线风格，例如 `max-value`。
    KebabCase,
}

impl VariableType {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 为路径类型补充“基于父目录创建子目录”的便捷能力。
pub trait ParentPathExt {
    /// 取当前路径的父目录并在其下创建 `child_path`，返回创建后的完整路径。
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>;
}

/// 模板格式化参数的弱类型载体。
///
/// `%s`、`%d`、`%f`、`%x` 等模板占位符会按目标格式读取这里的值；
/// 类型不匹配时采用宽松转换，无法解析的字符串按 `0` 处理。
#[derive(Clone, Debug, derive_more::From, PartialEq, derive_more::Display)]
pub enum FormatArg {
    /// 空值，占位格式化时显示为 `null` 或数值 `0`。
    #[from(skip)]
    #[display("null")]
    Null,
    /// 字符串参数。
    #[from(String, &str)]
    #[display("{_0}")]
    String(String),
    /// 有符号整数参数。
    #[from(i8, i16, i32, i64)]
    #[display("{_0}")]
    Integer(i64),
    /// 无符号整数参数。
    #[from(u8, u16, u32, u64)]
    #[display("{_0}")]
    Unsigned(u64),
    /// 浮点参数。
    #[from(f32, f64)]
    #[display("{_0}")]
    Float(f64),
    /// 布尔参数，数值格式化时 `true` 为 `1`、`false` 为 `0`。
    #[from(bool)]
    #[display("{_0}")]
    Boolean(bool),
}

/// Markdown 列表项标记处理方式。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkdownListMarkerMode {
    /// 保留 `- ` 和 `* ` 等列表标记。
    Keep,
    /// 去掉 `- ` 和 `* ` 等列表标记。
    Strip,
}

impl FormatArg {
    fn as_text(&self) -> String {
        self.to_string()
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

impl From<isize> for FormatArg {
    fn from(value: isize) -> Self {
        match value {
            value => FormatArg::Integer(value as i64),
        }
    }
}

impl From<usize> for FormatArg {
    fn from(value: usize) -> Self {
        match value {
            value => FormatArg::Unsigned(value as u64),
        }
    }
}

/// 清理输入两端和中间空白，并移除不可见控制字符。
///
/// `None` 和空串都会返回空 `String`；连续空白会折叠成单个空格。
pub fn clean_blank(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    if input.is_empty() {
        return String::new();
    }

    collapse_whitespace(input.trim())
        .chars()
        .filter(|character| is_visible(*character))
        .collect()
}

/// 取 `path` 的父目录，在父目录下创建 `child_path` 并返回创建后的路径。
///
/// 当 `path` 没有父目录时，直接创建并返回 `child_path`。
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

/// 按分隔项把线性列表切成多个段。
///
/// 满足 `predicate` 的元素会成为结果里的 key，直到下一个分隔项之前的元素会成为该 key 的值。
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

/// 确保输入同时带有指定前缀和后缀。
pub fn make_surround_with(input: Option<&str>, fix: &str) -> String {
    let with_prefix = add_prefix_if_not(input, fix, false);
    add_suffix_if_not(Some(&with_prefix), fix, false)
}

/// 确保非空文本被 `<p>` 和 `</p>` 包裹。
///
/// `None`、空串和纯空白输入都会返回空字符串。
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

/// 用英文圆括号包裹输入。
pub fn make_surround_with_brackets(input: &str) -> String {
    format!("({input})")
}

/// 只保留输入里的中文字符。
///
/// `None` 会返回空字符串；标点、数字、英文字母和其他字符都会被过滤。
pub fn remove_not_chinese(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    input
        .chars()
        .filter(|character| is_chinese(*character))
        .collect()
}

/// 当输入尚未以 `suffix` 结尾时追加后缀。
///
/// `ignore_case` 只影响后缀存在性判断，不改变返回文本本身的大小写。
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

/// 当输入尚未以 `prefix` 开头时追加前缀。
///
/// `None` 和空串返回空字符串；`ignore_case` 只影响前缀存在性判断。
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

/// 判断输入是否不是空白文本。
pub fn is_not_blank(input: Option<&str>) -> bool {
    !is_blank(input)
}

/// 返回 trim 后的非空文本切片。
///
/// `None`、空串和纯空白文本都会返回 `None`；返回值借用原始输入。
pub fn trim_non_blank(input: Option<&str>) -> Option<&str> {
    input.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

/// 返回 trim 后的非空自有字符串。
///
/// `None`、空串和纯空白文本都会返回 `None`；命中值会丢弃首尾空白。
pub fn trim_non_blank_owned(input: Option<String>) -> Option<String> {
    input.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

/// 归一化可选 ID 文本；为空时使用调用方提供的兜底值。
///
/// 该函数只处理字符串清洗，不绑定 UUID 或数据库策略，避免把 ID 生成职责塞进文本 crate。
pub fn normalized_id_or_else<F>(input: Option<String>, fallback: F) -> String
where
    F: FnOnce() -> String,
{
    trim_non_blank_owned(input).unwrap_or_else(fallback)
}

/// 按 Unicode 字符数量截断文本。
pub fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// 按 Unicode 字符数量截断文本，发生截断时追加 `…`。
pub fn truncate_chars_with_ellipsis(text: &str, limit: usize) -> String {
    let mut output = String::new();
    for (count, character) in text.chars().enumerate() {
        if count == limit {
            output.push('…');
            break;
        }
        output.push(character);
    }
    output
}

/// 提取 Markdown 中适合作为预览文本的纯文本行。
///
/// 该函数会跳过围栏代码块、空行和标题行，并将剩余行用单个空格连接。
pub fn clean_markdown_plain_text(input: &str, list_marker_mode: MarkdownListMarkerMode) -> String {
    let mut in_code_block = false;
    let mut lines = Vec::new();

    for raw in input.lines() {
        let line = raw.trim();
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = match list_marker_mode {
            MarkdownListMarkerMode::Keep => line,
            MarkdownListMarkerMode::Strip => line
                .strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line),
        };
        lines.push(line);
    }

    lines.join(" ")
}

/// 从点分路径右侧移除 `n` 段。
///
/// 当路径段数小于 `n` 时返回原始输入。
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

/// 将输入整体转成小写，`None` 返回空字符串。
pub fn lower_case(input: Option<&str>) -> String {
    input.unwrap_or_default().to_lowercase()
}

/// 只把首个 Unicode 字符转成小写，其余内容保持原样。
pub fn lower_first(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut result = first.to_lowercase().collect::<String>();
    result.push_str(chars.as_str());
    result
}

/// 判断 `value` 是否按 ASCII 忽略大小写存在于集合中。
pub fn ignore_case_in<I, S>(value: &str, collection: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    collection
        .into_iter()
        .any(|item| item.as_ref().eq_ignore_ascii_case(value))
}

/// 判断 `value` 是否按 ASCII 忽略大小写不存在于集合中。
pub fn ignore_case_not_in<I, S>(value: &str, collection: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    !ignore_case_in(value, collection)
}

/// 判断文本是否包含任意非空子串，比较时忽略大小写。
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

/// 判断 `value` 是否包含 `other`，比较时忽略大小写。
pub fn ignore_case_like(value: &str, other: &str) -> bool {
    value.to_lowercase().contains(&other.to_lowercase())
}

/// 判断输入中是否包含 CJK 统一表意文字范围内的中文字符。
pub fn contains_chinese(input: Option<&str>) -> bool {
    input.unwrap_or_default().chars().any(is_chinese)
}

/// 构造稳定的 ASCII 表名。
///
/// 优先使用英文名；英文名为空时使用中文名并经 `deunicode` 转写，随后规整成 `snake_case`。
pub fn default_table_english_name(english_name: &str, chinese_name: Option<&str>) -> String {
    let source = if english_name.trim().is_empty() {
        chinese_name.unwrap_or_default()
    } else {
        english_name
    };

    let without_parenthetical = remove_parenthetical(source);
    let ascii = deunicode(without_parenthetical.trim());
    let name = to_snake_case(&ascii, "", "");
    let compact = collapse_repeated_char(&name, '_');
    compact.trim_matches('_').to_owned()
}

/// 用指定分隔符拼接字符串切片。
pub fn join<S: AsRef<str>>(separator: &str, values: &[S]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(separator)
}

/// 把 Java/Kotlin 风格点分包名拼到基础路径后。
pub fn with_pkg(base: &str, pkg: &str) -> String {
    format!("{base}/{}", pkg.replace('.', "/"))
}

/// 把文件名拼到基础路径后。
pub fn with_file_name(base: &str, file_name: &str) -> String {
    format!("{base}/{file_name}")
}

/// 给基础路径追加文件后缀，未传后缀时默认追加 `.kt`。
pub fn with_file_suffix(base: &str, suffix: Option<&str>) -> String {
    format!("{base}{}", suffix.unwrap_or(".kt"))
}

/// 确保路径文本以 `/` 开头。
///
/// 输入会先去除首尾空白；空输入返回 `/`。
pub fn ensure_leading_slash(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with('/') {
        trimmed.to_owned()
    } else {
        format!("/{trimmed}")
    }
}

/// 归一化 URL path 文本。
///
/// 会移除 query 和 fragment，去除首尾空白，并保证结果以 `/` 开头；空路径返回 `/`。
pub fn normalize_url_path(input: &str) -> String {
    let without_hash = input.split('#').next().unwrap_or(input);
    let without_query = without_hash.split('?').next().unwrap_or(without_hash);
    let trimmed = without_query.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_owned()
    } else {
        format!("/{}", trimmed.trim_matches('/'))
    }
}

/// 把 URL path 拆成非空路径段。
///
/// 输入会先按 [`normalize_url_path`] 规整，根路径返回空列表。
pub fn split_url_path_segments(input: &str) -> Vec<String> {
    let normalized = normalize_url_path(input);
    if normalized == "/" {
        return Vec::new();
    }

    normalized
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// 从输入中移除给定集合里的所有字符串片段。
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

/// 移除双引号和反斜杠。
pub fn remove_any_quote(input: &str) -> String {
    remove_any(Some(input), ["\"", "\\"])
}

/// 移除空格和双引号。
pub fn remove_blank_or_quotation(input: &str) -> String {
    remove_any(Some(input), [" ", "\""])
}

/// 按标识符词边界转成下划线分隔形式，并保留原有大小写。
pub fn to_underline_case(input: &str) -> String {
    join_identifier_words(input, "_", CaseStyle::Preserve)
}

/// 按标识符词边界转成小写下划线形式。
pub fn to_underline_lower_case(input: &str) -> String {
    to_underline_case(input).to_lowercase()
}

/// 判断输入是否是带可选正负号和小数部分的十进制数字。
pub fn is_number(input: &str) -> bool {
    is_decimal_number(input)
}

/// 按 ASCII 忽略大小写比较两个字符串是否相等。
pub fn equals_ignore_case(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// 把可选值转成去空格、去双引号后的字符串。
pub fn to_not_empty_str<T: ToString>(value: Option<T>) -> String {
    value
        .map(|value| remove_blank_or_quotation(&value.to_string()))
        .unwrap_or_default()
}

/// 使用 `format_template` 的兼容入口。
pub fn kmp_format(template: &str, args: &[FormatArg]) -> String {
    format_template(template, args)
}

/// 使用固定小数位格式化 `f64`。
pub fn format_decimal(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// 使用固定小数位格式化 `f32`。
pub fn format_decimal_f32(value: f32, decimals: usize) -> String {
    format_decimal(value as f64, decimals)
}

/// 货币数值格式化入口，目前与固定小数位格式化保持一致。
pub fn format_currency(value: f64, decimals: usize) -> String {
    format_decimal(value, decimals)
}

/// `f32` 货币数值格式化入口。
pub fn format_currency_f32(value: f32, decimals: usize) -> String {
    format_currency(value as f64, decimals)
}

/// 基于 KMP 算法的可复用字符串匹配器。
///
/// 匹配位置以 UTF-8 字节偏移返回，适合继续用于字符串切片。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmpMatcher {
    pattern: String,
    lps: Vec<usize>,
}

impl KmpMatcher {
    /// 预计算模式串的 LPS 表，创建匹配器。
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let lps = compute_lps(&pattern);
        Self { pattern, lps }
    }

    /// 返回首个匹配位置；未命中时返回 `-1`。
    ///
    /// 空模式串按传统 `index_of` 语义返回 `0`。
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

    /// 返回所有匹配位置，支持重叠匹配。
    ///
    /// 空模式串返回空列表，避免产生无意义的每字节命中。
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

/// 判断文本中是否存在模式串。
pub fn contains_kmp(text: &str, pattern: &str) -> bool {
    KmpMatcher::new(pattern).search(text) != -1
}

/// 返回模式串首次出现的字节偏移；未命中时返回 `-1`。
pub fn index_of_kmp(text: &str, pattern: &str) -> isize {
    KmpMatcher::new(pattern).search(text)
}

/// 返回模式串所有出现位置的字节偏移。
pub fn find_all_kmp(text: &str, pattern: &str) -> Vec<usize> {
    KmpMatcher::new(pattern).search_all(text)
}

/// 使用 KMP 匹配结果替换文本。
///
/// 空模式串或未命中时返回原文本；重叠命中按从左到右的非重叠替换处理。
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

/// 移除目标字符最后一次出现的位置。
///
/// `None`、空串和纯空白输入返回空字符串；未找到目标字符时返回原文本。
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

/// 提取 Markdown fenced code block 中的正文内容。
///
/// 未检测到代码围栏时返回原文本；`None` 和空串返回空字符串。
pub fn extract_markdown_block_content(markdown: Option<&str>) -> String {
    let Some(markdown) = markdown else {
        return String::new();
    };
    if markdown.is_empty() {
        return String::new();
    }

    if markdown.contains("```") || markdown.contains("json") {
        let Some(regex) = fenced_block_regex() else {
            return String::new();
        };

        return regex
            .captures(markdown)
            .and_then(|captures| captures.get(1))
            .map(|matched| matched.as_str().trim().to_owned())
            .unwrap_or_default();
    }

    markdown.to_owned()
}

/// 提取双反引号包裹的代码块正文。
pub fn extract_code_block_content(code: impl AsRef<str>) -> String {
    let Some(regex) = double_tick_regex() else {
        return String::new();
    };

    regex
        .captures(code.as_ref())
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str().trim().to_owned())
        .unwrap_or_default()
}

/// 把任意输入规整成指定风格的有效变量名。
///
/// 会移除非法标识符字符；纯数字会加 `__` 前缀，以避免直接生成非法标识符。
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

    let mut cleaned = input
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || *character == '_'
                || *character == '-'
        })
        .collect::<String>();
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

/// 把输入规整成常量名风格。
pub fn to_constant_name(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::Constant, prefix, suffix)
}

/// 把输入规整成小驼峰变量名。
pub fn to_camel_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::CamelCase, prefix, suffix)
}

/// 把输入规整成大驼峰类型名。
pub fn to_pascal_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::PascalCase, prefix, suffix)
}

/// 把输入规整成小写下划线变量名。
pub fn to_snake_case(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::SnakeCase, prefix, suffix)
}

/// 把输入规整成短横线名称。
pub fn to_kebab_name(input: &str, prefix: &str, suffix: &str) -> String {
    to_valid_variable_name(input, VariableType::KebabCase, prefix, suffix)
}

/// 返回输入的 UTF-8 字节长度，`None` 返回 `0`。
pub fn length(input: Option<&str>) -> usize {
    input.map_or(0, str::len)
}

/// 判断输入是否包含任意给定子串。
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

/// 判断输入是否为 `None` 或 trim 后为空。
pub fn is_blank(input: Option<&str>) -> bool {
    input.is_none_or(|value| value.trim().is_empty())
}

/// 判断输入是否为 `None` 或原始字符串为空。
pub fn is_null_or_empty(input: Option<&str>) -> bool {
    input.is_none_or(str::is_empty)
}

/// 把连续重复出现的指定符号压缩为单个符号。
///
/// 任一输入为 `None` 时返回 `None`；重复符号为空时返回原文本。
pub fn remove_duplicate_symbol(
    source: Option<&str>,
    duplicate_element: Option<&str>,
) -> Option<String> {
    let source = source?;
    let duplicate_element = duplicate_element?;
    if duplicate_element.is_empty() {
        return Some(source.to_owned());
    }

    Some(collapse_repeated_str(source, duplicate_element))
}

/// 提取所有 `<p>...</p>` 标签之间的文本。
pub fn extract_text_between_p_tags(input: Option<&str>) -> Vec<String> {
    let Some(input) = input else {
        return Vec::new();
    };
    if !input.contains("<p>") || !input.contains("</p>") {
        return Vec::new();
    }

    let Some(regex) = p_tag_regex() else {
        return Vec::new();
    };

    regex
        .captures_iter(input)
        .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_owned()))
        .collect()
}

/// 清理块注释或文档注释标记，并折叠空白。
pub fn clean_doc_comment(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    collapse_whitespace(&remove_doc_comment_markers(input).replace('\n', " "))
        .trim()
        .to_owned()
}

/// 返回第一个 trim 后非空的字符串。
///
/// 保留命中值原始空白；全部为空时返回空字符串。
pub fn first_not_blank(values: &[Option<&str>]) -> String {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
        .unwrap_or_default()
}

/// 从完整 URL 中提取 `/api/...` 之后的 REST 路径。
pub fn get_rest_url(input: Option<&str>) -> String {
    let Some(input) = input else {
        return String::new();
    };
    let Some(regex) = rest_url_regex() else {
        return String::new();
    };

    regex
        .captures(input)
        .and_then(|captures| captures.get(2))
        .map(|value| value.as_str().to_owned())
        .unwrap_or_default()
}

/// 从文本中提取 `key: value` 或 `key：value` 形式的键值对。
pub fn extract_key_value_pairs(input: &str) -> HashMap<String, String> {
    let Some(regex) = key_value_regex() else {
        return HashMap::new();
    };

    regex
        .captures_iter(input)
        .filter_map(|captures| {
            let key = captures.get(1)?.as_str().trim().to_owned();
            let value = captures.get(2)?.as_str().trim().to_owned();
            Some((key, value))
        })
        .collect()
}

/// 转义常见 shell、HTML、SQL/注释和正则相关特殊字符。
///
/// 该函数用于降低把用户文本拼进命令、片段或展示文本时的误解释风险，不能替代上下文专用编码器。
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

/// 转义 XML 文本节点和属性值中的五个预定义实体。
pub fn escape_xml(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(character),
        }
    }
    output
}

/// 转义 SQL 单引号字符串字面量内部内容。
///
/// 只做 SQL 标准单引号加倍转义，不负责补外层单引号，也不替代表达式参数绑定。
pub fn escape_sql_string_literal(input: &str) -> String {
    input.replace('\'', "''")
}

/// 把文本包装成单引号 SQL 字符串字面量。
///
/// 该函数用于确实只能拼接 SQL 文本的底层边界；业务查询优先使用参数绑定。
pub fn quote_sql_string_literal(input: &str) -> String {
    format!("'{}'", escape_sql_string_literal(input))
}

/// 使用 POSIX shell 兼容的单引号形式包裹文本。
///
/// 单引号内部无法直接转义单引号，因此会拆成 `'\''` 片段。
pub fn quote_posix_shell_single(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    let mut output = String::with_capacity(input.len() + 2);
    output.push('\'');
    for character in input.chars() {
        if character == '\'' {
            output.push_str("'\\''");
        } else {
            output.push(character);
        }
    }
    output.push('\'');
    output
}

/// 解码常见 HTML 实体。
///
/// 该函数覆盖轻量元数据抓取场景中常见的 `&amp;`、`&quot;`、
/// `&#39;`、`&lt;`、`&gt;` 和 `&nbsp;`，不是完整 HTML5 实体解码器。
pub fn unescape_basic_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

/// 按标识符词边界转成小写短横线形式。
pub fn to_kebab_case(input: &str) -> String {
    join_identifier_words(input, "-", CaseStyle::Lower)
}

/// 按轻量 C 风格占位符格式化模板。
///
/// 支持 `%s`、`%d`、`%f`、`%x`、`%.Nf` 和 `%%`；参数不足时使用 `FormatArg::Null`。
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

/// 返回点分名称的最后一段。
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

fn p_tag_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?s)<p>(.*?)</p>").ok())
        .as_ref()
}

fn fenced_block_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?s)```[\w-]*\s*(.*?)\s*```").ok())
        .as_ref()
}

fn double_tick_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"(?s)``\w*\s*(.*?)\s*``").ok())
        .as_ref()
}

fn rest_url_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r".*:\d+(/[^/]+)(/.*)").ok())
        .as_ref()
}

fn key_value_regex() -> Option<&'static Regex> {
    static REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r"([\p{L}\p{N}_]+)[ \t]*[:：][ \t]*([\p{L}\p{N}_]+)").ok())
        .as_ref()
}

/// 将连续空白折叠为单个 ASCII 空格，并去除首尾空白。
pub fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_parenthetical(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut depth = 0usize;
    for character in input.chars() {
        match character {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => output.push(character),
            _ => {}
        }
    }
    output
}

fn collapse_repeated_char(input: &str, target: char) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_was_target = false;
    for character in input.chars() {
        if character == target {
            if !previous_was_target {
                output.push(character);
            }
            previous_was_target = true;
        } else {
            output.push(character);
            previous_was_target = false;
        }
    }
    output
}

fn collapse_repeated_str(input: &str, target: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(index) = rest.find(target) {
        output.push_str(&rest[..index]);
        output.push_str(target);
        rest = &rest[index + target.len()..];
        while rest.starts_with(target) {
            rest = &rest[target.len()..];
        }
    }
    output.push_str(rest);
    output
}

fn is_decimal_number(input: &str) -> bool {
    let input = input.trim();
    if input.is_empty() {
        return false;
    }
    let input = input
        .strip_prefix('-')
        .or_else(|| input.strip_prefix('+'))
        .unwrap_or(input);
    let mut seen_digit = false;
    let mut seen_dot = false;
    for character in input.chars() {
        if character.is_ascii_digit() {
            seen_digit = true;
        } else if character == '.' && !seen_dot {
            seen_dot = true;
        } else {
            return false;
        }
    }
    seen_digit
}

fn remove_doc_comment_markers(input: &str) -> String {
    input
        .replace("/**", " ")
        .replace("/*", " ")
        .replace("*/", " ")
        .replace(['*', '/'], " ")
}
