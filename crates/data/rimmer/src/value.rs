use serde::{Deserialize, Serialize};

/// SQL 参数标量值。
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarValue {
    /// SQL NULL。
    Null,
    /// 文本值。
    Text(String),
    /// 有符号整数。
    I64(i64),
    /// 无符号整数。
    U64(u64),
    /// 浮点数。
    F64(f64),
    /// 布尔值。
    Bool(bool),
}

/// 转换为 SQL 参数标量值。
pub trait ToScalarValue {
    /// 执行转换。
    fn to_scalar_value(self) -> ScalarValue;
}

impl ToScalarValue for ScalarValue {
    fn to_scalar_value(self) -> ScalarValue {
        self
    }
}

impl ToScalarValue for String {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::Text(self)
    }
}

impl ToScalarValue for &str {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::Text(self.to_string())
    }
}

impl ToScalarValue for bool {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::Bool(self)
    }
}

impl ToScalarValue for i8 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::I64(i64::from(self))
    }
}

impl ToScalarValue for i16 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::I64(i64::from(self))
    }
}

impl ToScalarValue for i32 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::I64(i64::from(self))
    }
}

impl ToScalarValue for i64 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::I64(self)
    }
}

impl ToScalarValue for u8 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::U64(u64::from(self))
    }
}

impl ToScalarValue for u16 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::U64(u64::from(self))
    }
}

impl ToScalarValue for u32 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::U64(u64::from(self))
    }
}

impl ToScalarValue for u64 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::U64(self)
    }
}

impl ToScalarValue for f32 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::F64(f64::from(self))
    }
}

impl ToScalarValue for f64 {
    fn to_scalar_value(self) -> ScalarValue {
        ScalarValue::F64(self)
    }
}

impl<T> ToScalarValue for Option<T>
where
    T: ToScalarValue,
{
    fn to_scalar_value(self) -> ScalarValue {
        match self {
            Some(value) => value.to_scalar_value(),
            None => ScalarValue::Null,
        }
    }
}
