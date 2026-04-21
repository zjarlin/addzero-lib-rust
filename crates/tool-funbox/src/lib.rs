#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const STRING_LENGTH_DEFAULT: &str = "255";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDto {
    pub rest_name: Option<String>,
    pub rest_url: Option<String>,
    pub model_name: Option<String>,
    pub field_name: Option<String>,
    pub field_eng: Option<String>,
    pub field_type: Option<String>,
    pub field_long: Option<String>,
}

pub type FieldDTO = FieldDto;

impl FieldDto {
    pub fn builder() -> FieldDtoBuilder {
        FieldDtoBuilder::default()
    }

    pub fn string_field(field_name: impl Into<String>, field_eng: impl Into<String>) -> Self {
        Self::builder()
            .field_name(field_name)
            .field_eng(field_eng)
            .field_type("String")
            .field_long(STRING_LENGTH_DEFAULT)
            .build()
    }

    pub fn is_empty(&self) -> bool {
        self.rest_name.is_none()
            && self.rest_url.is_none()
            && self.model_name.is_none()
            && self.field_name.is_none()
            && self.field_eng.is_none()
            && self.field_type.is_none()
            && self.field_long.is_none()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldDtoBuilder {
    rest_name: Option<String>,
    rest_url: Option<String>,
    model_name: Option<String>,
    field_name: Option<String>,
    field_eng: Option<String>,
    field_type: Option<String>,
    field_long: Option<String>,
}

impl FieldDtoBuilder {
    pub fn rest_name(mut self, value: impl Into<String>) -> Self {
        self.rest_name = Some(value.into());
        self
    }

    pub fn rest_url(mut self, value: impl Into<String>) -> Self {
        self.rest_url = Some(value.into());
        self
    }

    pub fn model_name(mut self, value: impl Into<String>) -> Self {
        self.model_name = Some(value.into());
        self
    }

    pub fn field_name(mut self, value: impl Into<String>) -> Self {
        self.field_name = Some(value.into());
        self
    }

    pub fn field_eng(mut self, value: impl Into<String>) -> Self {
        self.field_eng = Some(value.into());
        self
    }

    pub fn field_type(mut self, value: impl Into<String>) -> Self {
        self.field_type = Some(value.into());
        self
    }

    pub fn field_long(mut self, value: impl Into<String>) -> Self {
        self.field_long = Some(value.into());
        self
    }

    pub fn build(self) -> FieldDto {
        FieldDto {
            rest_name: self.rest_name,
            rest_url: self.rest_url,
            model_name: self.model_name,
            field_name: self.field_name,
            field_eng: self.field_eng,
            field_type: self.field_type,
            field_long: self.field_long,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunBox {
    pub rest_url: Option<String>,
    pub method_type: Option<String>,
    pub des: Option<String>,
    pub fun_name: Option<String>,
    #[serde(default)]
    pub paramiter: Vec<FieldDto>,
    #[serde(default)]
    pub returns: Vec<FieldDto>,
}

impl FunBox {
    pub fn builder() -> FunBoxBuilder {
        FunBoxBuilder::default()
    }

    pub fn parameters(&self) -> &[FieldDto] {
        &self.paramiter
    }

    pub fn has_parameters(&self) -> bool {
        !self.paramiter.is_empty()
    }

    pub fn has_returns(&self) -> bool {
        !self.returns.is_empty()
    }

    pub fn signature(&self) -> String {
        let method = self.method_type.as_deref().unwrap_or("UNKNOWN");
        let path = self.rest_url.as_deref().unwrap_or("");
        let name = self.fun_name.as_deref().unwrap_or("");
        format!("{method} {path} {name}").trim().to_owned()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunBoxBuilder {
    rest_url: Option<String>,
    method_type: Option<String>,
    des: Option<String>,
    fun_name: Option<String>,
    paramiter: Vec<FieldDto>,
    returns: Vec<FieldDto>,
}

impl FunBoxBuilder {
    pub fn rest_url(mut self, value: impl Into<String>) -> Self {
        self.rest_url = Some(value.into());
        self
    }

    pub fn method_type(mut self, value: impl Into<String>) -> Self {
        self.method_type = Some(value.into());
        self
    }

    pub fn des(mut self, value: impl Into<String>) -> Self {
        self.des = Some(value.into());
        self
    }

    pub fn fun_name(mut self, value: impl Into<String>) -> Self {
        self.fun_name = Some(value.into());
        self
    }

    pub fn parameter(mut self, value: FieldDto) -> Self {
        self.paramiter.push(value);
        self
    }

    pub fn parameters<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = FieldDto>,
    {
        self.paramiter.extend(values);
        self
    }

    pub fn return_field(mut self, value: FieldDto) -> Self {
        self.returns.push(value);
        self
    }

    pub fn returns<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = FieldDto>,
    {
        self.returns.extend(values);
        self
    }

    pub fn build(self) -> FunBox {
        FunBox {
            rest_url: self.rest_url,
            method_type: self.method_type,
            des: self.des,
            fun_name: self.fun_name,
            paramiter: self.paramiter,
            returns: self.returns,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunBoxRegistry {
    entries: Vec<FunBox>,
}

impl FunBoxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, fun_box: FunBox) -> &mut Self {
        self.entries.push(fun_box);
        self
    }

    pub fn extend<I>(&mut self, fun_boxes: I) -> &mut Self
    where
        I: IntoIterator<Item = FunBox>,
    {
        self.entries.extend(fun_boxes);
        self
    }

    pub fn all(&self) -> &[FunBox] {
        &self.entries
    }

    pub fn into_all(self) -> Vec<FunBox> {
        self.entries
    }

    pub fn find_by_fun_name(&self, fun_name: &str) -> Option<&FunBox> {
        self.entries
            .iter()
            .find(|item| item.fun_name.as_deref() == Some(fun_name))
    }

    pub fn find_by_rest_url(&self, rest_url: &str) -> Vec<&FunBox> {
        self.entries
            .iter()
            .filter(|item| item.rest_url.as_deref() == Some(rest_url))
            .collect()
    }

    pub fn find_by_method_type(&self, method_type: &str) -> Vec<&FunBox> {
        self.entries
            .iter()
            .filter(|item| {
                item.method_type
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(method_type))
            })
            .collect()
    }
}

pub struct AbsFunBox;

impl AbsFunBox {
    pub fn get_all_fun(registry: &FunBoxRegistry) -> Vec<FunBox> {
        registry.all().to_vec()
    }
}
