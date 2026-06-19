//! YAML 点号路径解析与查询。

use anyhow::{Context, Result, bail};
use serde_yaml::Value;

use crate::env_subst::env_subst;

/// YAML 路径中的一个访问片段。
///
/// `a.b[0]` 会被拆成两个键片段和一个下标片段，供 [`YamlDoc`] 在 `serde_yaml::Value` 树上逐层查找。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum YamlPathSegment {
    /// 映射键访问，支持普通点号键和方括号中的带引号键。
    Key(String),
    /// 序列下标访问。
    Index(usize),
}

/// 已解析的 YAML 点号路径。
///
/// 路径语法面向配置读取，不尝试覆盖完整 JSONPath；支持 `spring.datasource.url`、
/// `items[0].name` 和 `spring.datasource["jdbc-url"]` 这类常见 Spring 配置路径。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YamlPath {
    segments: Vec<YamlPathSegment>,
}

impl YamlPath {
    /// 解析字符串路径为可复用的 `YamlPath`。
    ///
    /// 空路径、未闭合方括号、非法分隔符等语法问题会返回 `anyhow::Error`。
    pub fn parse(path: impl AsRef<str>) -> Result<Self> {
        let input = path.as_ref().trim();
        if input.is_empty() {
            bail!("yaml path is invalid: path cannot be empty");
        }

        let chars: Vec<char> = input.chars().collect();
        let mut index = 0usize;
        let mut segments = Vec::new();

        while index < chars.len() {
            skip_whitespace(&chars, &mut index);

            if index < chars.len() && chars[index] == '.' {
                index += 1;
                skip_whitespace(&chars, &mut index);
                if index >= chars.len() {
                    let message = format!("path `{input}` cannot end with `.`");
                    bail!("yaml path is invalid: {message}");
                }
            }

            if index >= chars.len() {
                break;
            }

            if chars[index] == '[' {
                segments.push(parse_bracket_segment(&chars, &mut index, input)?);
            } else {
                segments.push(parse_bare_segment(&chars, &mut index, input)?);
            }

            skip_whitespace(&chars, &mut index);
            if index < chars.len() && chars[index] != '.' && chars[index] != '[' {
                let message = format!("unexpected character `{}` in path `{input}`", chars[index]);
                bail!("yaml path is invalid: {message}");
            }
        }

        Ok(Self { segments })
    }

    /// 返回解析后的路径片段。
    ///
    /// 片段按访问顺序排列，调用方可以用于自定义 `serde_yaml::Value` 遍历逻辑。
    pub fn segments(&self) -> &[YamlPathSegment] {
        &self.segments
    }
}

impl std::str::FromStr for YamlPath {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(value)
    }
}

/// YAML 文档值包装器。
///
/// 该类型不改变原始 `serde_yaml::Value` 的结构，只在路径查询和字符串读取时提供更贴近配置文件的便捷语义。
#[derive(Clone, Debug, PartialEq)]
pub struct YamlDoc {
    value: Value,
}

impl YamlDoc {
    /// 用已经解析好的 `serde_yaml::Value` 构造文档包装器。
    pub fn from_value(value: Value) -> Self {
        Self { value }
    }

    /// 借用底层 YAML 值。
    pub fn as_value(&self) -> &Value {
        &self.value
    }

    /// 消耗包装器并取回底层 YAML 值。
    pub fn into_inner(self) -> Value {
        self.value
    }

    /// 使用已经解析好的路径查询原始 YAML 值。
    pub fn get_path(&self, path: &YamlPath) -> Option<&Value> {
        lookup_value(&self.value, path)
    }

    /// 解析字符串路径并查询原始 YAML 值。
    ///
    /// 路径语法错误会返回 `anyhow::Error`；路径合法但值不存在时返回 `Ok(None)`。
    pub fn get(&self, path: &str) -> Result<Option<&Value>> {
        let parsed = YamlPath::parse(path)?;
        Ok(self.get_path(&parsed))
    }

    /// 解析字符串路径并把标量值转换为字符串。
    ///
    /// 仅字符串、数字和布尔值会被转换；映射和序列返回 `None`。返回字符串会经过 `${VAR:default}` 环境变量替换。
    pub fn get_string(&self, path: &str) -> Result<Option<String>> {
        let parsed = YamlPath::parse(path)?;
        Ok(self.get_string_at(&parsed))
    }

    /// 使用已解析路径读取字符串标量。
    ///
    /// 该方法适合在循环或多次读取中复用 `YamlPath`，避免重复解析路径文本。
    pub fn get_string_at(&self, path: &YamlPath) -> Option<String> {
        self.get_path(path)
            .and_then(stringify_scalar)
            .map(|value| env_subst(&value))
    }
}

/// 抽象 YAML 路径查找能力。
///
/// 该 trait 让 [`YamlDoc`]、原始 `serde_yaml::Value` 和它们的引用可以共享 `yaml_get!` 等便捷入口。
pub trait YamlLookup {
    /// 使用已解析路径查找 YAML 值。
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value>;
}

impl YamlLookup for YamlDoc {
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        self.get_path(path)
    }
}

impl YamlLookup for Value {
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        lookup_value(self, path)
    }
}

impl<T> YamlLookup for &T
where
    T: YamlLookup + ?Sized,
{
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        (*self).yaml_lookup(path)
    }
}

/// 在任意实现了 [`YamlLookup`] 的文档上查询路径。
///
/// 这是 `yaml_get!` 宏背后的公开函数，适合需要先缓存 [`YamlPath`] 的调用场景。
pub fn get_yaml_path_value<'a, T>(doc: &'a T, path: &YamlPath) -> Option<&'a Value>
where
    T: YamlLookup + ?Sized,
{
    doc.yaml_lookup(path)
}

fn skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

fn parse_bare_segment(
    chars: &[char],
    index: &mut usize,
    original: &str,
) -> Result<YamlPathSegment> {
    let mut segment = String::new();
    while *index < chars.len() && chars[*index] != '.' && chars[*index] != '[' {
        segment.push(chars[*index]);
        *index += 1;
    }

    let trimmed = segment.trim();
    if trimmed.is_empty() {
        let message = format!("empty segment in path `{original}`");
        bail!("yaml path is invalid: {message}");
    }

    Ok(YamlPathSegment::Key(trimmed.to_owned()))
}

fn parse_bracket_segment(
    chars: &[char],
    index: &mut usize,
    original: &str,
) -> Result<YamlPathSegment> {
    *index += 1;
    skip_whitespace(chars, index);

    if *index >= chars.len() {
        let message = format!("unclosed bracket in path `{original}`");
        bail!("yaml path is invalid: {message}");
    }

    let segment = if matches!(chars[*index], '"' | '\'') {
        let quote = chars[*index];
        *index += 1;
        let mut value = String::new();
        let mut closed = false;

        while *index < chars.len() {
            let current = chars[*index];
            if current == '\\' {
                *index += 1;
                if *index < chars.len() {
                    value.push(chars[*index]);
                    *index += 1;
                }
                continue;
            }

            if current == quote {
                *index += 1;
                closed = true;
                break;
            }

            value.push(current);
            *index += 1;
        }

        if !closed {
            let message = format!("unclosed quoted segment in path `{original}`");
            bail!("yaml path is invalid: {message}");
        }

        YamlPathSegment::Key(value)
    } else {
        let mut raw = String::new();
        while *index < chars.len() && chars[*index] != ']' {
            raw.push(chars[*index]);
            *index += 1;
        }

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            let message = format!("empty bracket segment in path `{original}`");
            bail!("yaml path is invalid: {message}");
        }

        if trimmed.chars().all(|character| character.is_ascii_digit()) {
            let value = trimmed.parse::<usize>().with_context(|| {
                format!("yaml path is invalid: invalid sequence index `{trimmed}` in `{original}`")
            })?;
            YamlPathSegment::Index(value)
        } else {
            YamlPathSegment::Key(trimmed.to_owned())
        }
    };

    skip_whitespace(chars, index);
    if *index >= chars.len() || chars[*index] != ']' {
        let message = format!("missing closing `]` in path `{original}`");
        bail!("yaml path is invalid: {message}");
    }
    *index += 1;

    Ok(segment)
}

fn lookup_value<'a>(root: &'a Value, path: &YamlPath) -> Option<&'a Value> {
    let mut current = root;

    for segment in path.segments() {
        current = match segment {
            YamlPathSegment::Key(key) => {
                let mapping = current.as_mapping()?;
                let key = Value::String(key.clone());
                mapping.get(&key)?
            }
            YamlPathSegment::Index(index) => current.as_sequence()?.get(*index)?,
        };
    }

    Some(current)
}

fn stringify_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(inner) => Some(inner.clone()),
        Value::Number(inner) => Some(inner.to_string()),
        Value::Bool(inner) => Some(inner.to_string()),
        _ => None,
    }
}

/// 构造 [`YamlPath`] 的便捷宏。
///
/// 字面量或 token 路径会在运行到宏展开代码时解析；路径非法时 panic，适合测试、常量式配置路径和确定不会来自用户输入的场景。
#[macro_export]
macro_rules! yaml_path {
    ($path:literal) => {{
        match <$crate::path::YamlPath as ::std::str::FromStr>::from_str($path) {
            Ok(path) => path,
            Err(error) => panic!("yaml_path!: invalid path literal: {error}"),
        }
    }};
    ($($path:tt)+) => {{
        match <$crate::path::YamlPath as ::std::str::FromStr>::from_str(::core::stringify!($($path)+)) {
            Ok(path) => path,
            Err(error) => panic!("yaml_path!: invalid path tokens: {error}"),
        }
    }};
}

/// 使用路径宏直接查询 YAML 文档。
///
/// 该宏组合 `yaml_path!` 与 [`get_yaml_path_value`]，返回 `Option<&serde_yaml::Value>`，不会执行字符串标量转换。
#[macro_export]
macro_rules! yaml_get {
    ($doc:expr, $path:literal) => {{
        let __path = $crate::yaml_path!($path);
        $crate::path::get_yaml_path_value(&$doc, &__path)
    }};
    ($doc:expr, $($path:tt)+) => {{
        let __path = $crate::yaml_path!($($path)+);
        $crate::path::get_yaml_path_value(&$doc, &__path)
    }};
}
