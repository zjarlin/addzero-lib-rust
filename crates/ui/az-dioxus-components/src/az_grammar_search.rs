use dioxus::prelude::*;

use crate::class_name::compose_class;

/// 描述语法式搜索框可提示的一个过滤维度。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AzGrammarSearchField {
    /// 过滤维度的输入键，例如 `tag` 或 `def`。
    pub key: String,
    /// 展示给用户看的短标签。
    pub label: String,
}

impl AzGrammarSearchField {
    /// 创建一个可展示在 `AzGrammarSearchInput` 中的过滤维度提示。
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// 解析后的语法式搜索查询。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GrammarSearchQuery {
    /// 原始输入文本。
    pub raw: String,
    /// 没有 `key:` 前缀的普通关键词。
    pub terms: Vec<GrammarSearchTerm>,
    /// 形如 `key:value` 的过滤条件。
    pub filters: Vec<GrammarSearchFilter>,
}

impl GrammarSearchQuery {
    /// 判断查询是否不包含关键词和过滤条件。
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty() && self.filters.is_empty()
    }

    /// 返回指定过滤键下的所有值，键比较不区分 ASCII 大小写。
    pub fn values_for(&self, key: &str) -> Vec<&str> {
        self.filters
            .iter()
            .filter(|filter| filter.key.eq_ignore_ascii_case(key))
            .flat_map(|filter| filter.values.iter().map(String::as_str))
            .collect()
    }
}

/// 解析后的普通关键词。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrammarSearchTerm {
    /// 关键词文本。
    pub value: String,
}

/// 解析后的 `key:value` 过滤条件。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrammarSearchFilter {
    /// 过滤键，例如 `tag`、`language`、`stars`。
    pub key: String,
    /// 过滤值。逗号会被解析成多个候选值。
    pub values: Vec<String>,
}

/// 解析 GitHub 风格的语法式搜索文本。
///
/// 支持空格或分号分隔条件，支持 `tag:rust,java` 多值，保留
/// `stars:>100` 这类比较表达式作为普通过滤值。
pub fn parse_grammar_search_query(input: &str) -> GrammarSearchQuery {
    let mut query = GrammarSearchQuery {
        raw: input.to_string(),
        ..GrammarSearchQuery::default()
    };

    for clause in split_preserving_quotes(input, &[';', '；']) {
        for token in split_preserving_quotes(&clause, &[' ', '\t', '\n', '\r']) {
            let token = unquote(token.trim());
            if token.is_empty() {
                continue;
            }

            if let Some((key, value)) = parse_filter_token(&token) {
                let values = split_values(value)
                    .into_iter()
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>();
                if !values.is_empty() {
                    query.filters.push(GrammarSearchFilter {
                        key: key.to_ascii_lowercase(),
                        values,
                    });
                }
            } else {
                query.terms.push(GrammarSearchTerm { value: token });
            }
        }
    }

    query
}

/// 渲染一个支持 `key:value` 语法和过滤 chip 的搜索输入框。
#[allow(non_snake_case)]
#[component]
pub fn AzGrammarSearchInput(
    value: String,
    oninput: EventHandler<String>,
    #[props(default, into)] class: String,
    #[props(default, into)] placeholder: String,
    #[props(default)] fields: Vec<AzGrammarSearchField>,
) -> Element {
    let root_class = compose_class("az-grammar-search", &class, &[]);
    let parsed_query = parse_grammar_search_query(&value);
    let filters = parsed_query.filters;
    let terms = parsed_query.terms;

    rsx! {
        div { class: root_class,
            div { class: "az-grammar-search__box",
                span { class: "az-grammar-search__icon", "⌕" }
                input {
                    class: "az-grammar-search__input",
                    value: "{value}",
                    placeholder: "{placeholder}",
                    oninput: move |event| oninput.call(event.value()),
                }
            }
            if !filters.is_empty() || !terms.is_empty() {
                div { class: "az-grammar-search__tokens",
                    for term in terms {
                        span { class: "az-grammar-search__token az-grammar-search__token--term",
                            "{term.value}"
                        }
                    }
                    for filter in filters {
                        for filter_value in filter.values {
                            span { class: "az-grammar-search__token",
                                span { class: "az-grammar-search__token-key", "{filter.key}" }
                                span { class: "az-grammar-search__token-separator", ":" }
                                span { class: "az-grammar-search__token-value", "{filter_value}" }
                            }
                        }
                    }
                }
            }
            if !fields.is_empty() {
                div { class: "az-grammar-search__fields",
                    for field in fields {
                        span { class: "az-grammar-search__field",
                            span { class: "az-grammar-search__field-key", "{field.key}" }
                            span { class: "az-grammar-search__field-label", "{field.label}" }
                        }
                    }
                }
            }
        }
    }
}

fn parse_filter_token(token: &str) -> Option<(&str, &str)> {
    let (key, value) = token.split_once(':')?;
    let key = key.trim();
    let value = value.trim();
    (!key.is_empty() && is_filter_key(key) && !value.is_empty()).then_some((key, value))
}

fn is_filter_key(key: &str) -> bool {
    key.chars()
        .all(|char| char == '_' || char == '-' || char.is_ascii_alphanumeric())
}

fn split_values(input: &str) -> Vec<String> {
    split_preserving_quotes(input, &[',', '，'])
        .into_iter()
        .map(|value| unquote(value.trim()))
        .collect()
}

fn split_preserving_quotes(input: &str, separators: &[char]) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for char in input.chars() {
        if escaped {
            current.push(char);
            escaped = false;
            continue;
        }

        if char == '\\' {
            current.push(char);
            escaped = true;
            continue;
        }

        match quote {
            Some(active_quote) if char == active_quote => {
                current.push(char);
                quote = None;
            }
            Some(_) => current.push(char),
            None if char == '\'' || char == '"' => {
                current.push(char);
                quote = Some(char);
            }
            None if separators.contains(&char) => {
                push_trimmed_part(&mut parts, &mut current);
            }
            None => current.push(char),
        }
    }

    push_trimmed_part(&mut parts, &mut current);
    parts
}

fn push_trimmed_part(parts: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    current.clear();
}

fn unquote(input: &str) -> String {
    let input = input.trim();
    if input.len() < 2 {
        return input.to_string();
    }

    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let Some(last) = input.chars().last() else {
        return String::new();
    };

    if (first == '"' || first == '\'') && first == last {
        input[first.len_utf8()..input.len() - last.len_utf8()].to_string()
    } else {
        input.to_string()
    }
}
