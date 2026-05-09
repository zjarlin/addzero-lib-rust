use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn from_json_str(input: &str) -> Result<Self, DictSpecError> {
        let spec = serde_json::from_str::<Self>(input)?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn to_pretty_json_string(&self) -> Result<String, DictSpecError> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn validate(&self) -> Result<(), DictSpecError> {
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
            return Err(DictSpecError::Validation(
                "items cannot be empty".to_string(),
            ));
        }

        let mut item_codes = BTreeSet::new();
        let mut int_values = BTreeSet::new();
        let mut text_values = BTreeSet::new();
        for item in &self.items {
            item.validate(self.raw_value_kind)?;
            if !item_codes.insert(item.code.clone()) {
                return Err(DictSpecError::Validation(format!(
                    "duplicate item code: {}",
                    item.code
                )));
            }
            match self.raw_value_kind {
                RawValueKind::Int => {
                    let value = item.raw_int_value.expect("validated raw_int_value");
                    if !int_values.insert(value) {
                        return Err(DictSpecError::Validation(format!(
                            "duplicate rawIntValue: {value}"
                        )));
                    }
                }
                RawValueKind::String => {
                    let value = item
                        .raw_text_value
                        .as_deref()
                        .expect("validated raw_text_value");
                    if !text_values.insert(value.to_string()) {
                        return Err(DictSpecError::Validation(format!(
                            "duplicate rawTextValue: {value}"
                        )));
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub fn validate(&self, raw_value_kind: RawValueKind) -> Result<(), DictSpecError> {
        ensure_non_empty("item.code", &self.code)?;
        ensure_non_empty("item.label", &self.label)?;
        match raw_value_kind {
            RawValueKind::Int => {
                if self.raw_int_value.is_none() || self.raw_text_value.is_some() {
                    return Err(DictSpecError::Validation(format!(
                        "item {} must define rawIntValue only",
                        self.code
                    )));
                }
            }
            RawValueKind::String => {
                if self.raw_int_value.is_some() || self.raw_text_value.is_none() {
                    return Err(DictSpecError::Validation(format!(
                        "item {} must define rawTextValue only",
                        self.code
                    )));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RawValueKind {
    Int,
    String,
}

#[derive(Debug, thiserror::Error)]
pub enum DictSpecError {
    #[error("invalid dictionary spec: {0}")]
    Validation(String),
    #[error("invalid dictionary spec json: {0}")]
    Json(#[from] serde_json::Error),
}

fn default_true() -> bool {
    true
}

fn ensure_non_empty(field: &str, value: &str) -> Result<(), DictSpecError> {
    if value.trim().is_empty() {
        return Err(DictSpecError::Validation(format!(
            "{field} cannot be empty"
        )));
    }
    Ok(())
}
