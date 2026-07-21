//! 字典（枚举）数据的规范定义与校验。
//!
//! 本 crate 提供 [`DictionarySpec`] 和 [`DictionaryItemSpec`] 结构体，用于在应用层统一描述、
//! 校验和序列化字典型数据（如状态码、分类标签、枚举选项等）。
//!
//! # 核心类型
//!
//! - [`DictionarySpec`] — 字典规范的顶层定义，包含代码、名称、作用域、原始值类型和条目列表
//! - [`DictionaryItemSpec`] — 单个字典条目的规范
//! - [`DictEnumItem<T>`] — 编译期静态字典枚举项，适用于代码内嵌常量
//! - [`RawValueKind`] — 原始值类型枚举（整数/字符串）
//!
//! # 校验规则
//!
//! - `code`、`name`、`scope`、`item.code`、`item.label` 不能为空
//! - `items` 列表不能为空
//! - 条目的 `code` 和原始值（`rawIntValue` / `rawTextValue`）不能重复
//! - 整数型字典只能使用 `rawIntValue`，字符串型字典只能使用 `rawTextValue`

use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DictEnumItem<T>
where
    T: Copy + 'static,
{
    pub code: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub raw_value: T,
    pub meta_json: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionarySpec {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub raw_value_kind: RawValueKind,
    #[serde(default)]
    pub open_enum: bool,
    pub unknown_variant: Option<String>,
    #[serde(default)]
    pub sort_index: i64,
    #[serde(default)]
    pub items: Vec<DictionaryItemSpec>,
}

impl DictionarySpec {
    pub fn from_json_str(input: &str) -> Result<Self> {
        let spec = serde_json::from_str::<Self>(input)?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn to_pretty_json_string(&self) -> Result<String> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("code", &self.code)?;
        ensure_non_empty("name", &self.name)?;
        ensure_non_empty("scope", &self.scope)?;
        if self.open_enum {
            ensure_non_empty(
                "unknownVariant",
                self.unknown_variant.as_deref().unwrap_or("Other"),
            )?;
        }
        if self.items.is_empty() {
            bail!("invalid dictionary spec: items cannot be empty");
        }

        let mut item_codes = BTreeSet::new();
        let mut int_values = BTreeSet::new();
        let mut text_values = BTreeSet::new();
        for item in &self.items {
            item.validate(self.raw_value_kind)?;
            if !item_codes.insert(item.code.clone()) {
                let message = format!("duplicate item code: {}", item.code);
                bail!("invalid dictionary spec: {message}");
            }
            match self.raw_value_kind {
                RawValueKind::Int => {
                    let Some(value) = item.raw_int_value else {
                        let message = format!("item {} must define rawIntValue", item.code);
                        bail!("invalid dictionary spec: {message}");
                    };
                    if !int_values.insert(value) {
                        let message = format!("duplicate rawIntValue: {value}");
                        bail!("invalid dictionary spec: {message}");
                    }
                }
                RawValueKind::String => {
                    let Some(value) = item.raw_text_value.as_deref() else {
                        let message = format!("item {} must define rawTextValue", item.code);
                        bail!("invalid dictionary spec: {message}");
                    };
                    if !text_values.insert(value.to_string()) {
                        let message = format!("duplicate rawTextValue: {value}");
                        bail!("invalid dictionary spec: {message}");
                    }
                }
            }
        }

        Ok(())
    }

    pub fn normalized_unknown_variant(&self) -> &str {
        self.unknown_variant
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Other")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryItemSpec {
    pub code: String,
    pub label: String,
    pub description: Option<String>,
    pub raw_int_value: Option<i64>,
    pub raw_text_value: Option<String>,
    #[serde(default)]
    pub sort_index: i64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub meta: Option<Value>,
}

impl DictionaryItemSpec {
    pub fn validate(&self, raw_value_kind: RawValueKind) -> Result<()> {
        ensure_non_empty("item.code", &self.code)?;
        ensure_non_empty("item.label", &self.label)?;
        match raw_value_kind {
            RawValueKind::Int => {
                if self.raw_int_value.is_none() || self.raw_text_value.is_some() {
                    let message = format!("item {} must define rawIntValue only", self.code);
                    bail!("invalid dictionary spec: {message}");
                }
            }
            RawValueKind::String => {
                if self.raw_int_value.is_some() || self.raw_text_value.is_none() {
                    let message = format!("item {} must define rawTextValue only", self.code);
                    bail!("invalid dictionary spec: {message}");
                }
                ensure_non_empty(
                    "item.rawTextValue",
                    self.raw_text_value.as_deref().unwrap_or_default(),
                )?;
            }
        }
        Ok(())
    }

    pub fn description_text(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RawValueKind {
    Int,
    String,
}

impl RawValueKind {
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

fn default_true() -> bool {
    true
}

fn ensure_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        let message = format!("{field} cannot be empty");
        bail!("invalid dictionary spec: {message}");
    }
    Ok(())
}
