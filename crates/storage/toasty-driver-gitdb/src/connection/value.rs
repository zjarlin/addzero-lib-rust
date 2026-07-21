use anyhow::{anyhow, bail};
use serde_json::Value as JsonValue;
use toasty_core::stmt::{self, Value};

pub(crate) fn to_json_value(value: &Value) -> anyhow::Result<JsonValue> {
    match value {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(v) => Ok(JsonValue::Bool(*v)),
        Value::I8(v) => Ok(JsonValue::from(*v)),
        Value::I16(v) => Ok(JsonValue::from(*v)),
        Value::I32(v) => Ok(JsonValue::from(*v)),
        Value::I64(v) => Ok(JsonValue::from(*v)),
        Value::U8(v) => Ok(JsonValue::from(*v)),
        Value::U16(v) => Ok(JsonValue::from(*v)),
        Value::U32(v) => Ok(JsonValue::from(*v)),
        Value::U64(v) => Ok(JsonValue::from(*v)),
        Value::F32(v) => serde_json::Number::from_f64((*v).into())
            .map(JsonValue::Number)
            .ok_or_else(|| anyhow!("unsupported gitdb/toasty value conversion: non-finite f32")),
        Value::F64(v) => serde_json::Number::from_f64(*v)
            .map(JsonValue::Number)
            .ok_or_else(|| anyhow!("unsupported gitdb/toasty value conversion: non-finite f64")),
        Value::String(v) => Ok(JsonValue::String(v.clone())),
        Value::Bytes(v) => Ok(JsonValue::Array(
            v.iter().copied().map(JsonValue::from).collect(),
        )),
        Value::Uuid(v) => Ok(JsonValue::String(v.to_string())),
        Value::List(values) => Ok(JsonValue::Array(
            values
                .iter()
                .map(to_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Value::Record(record) => Ok(JsonValue::Array(
            record
                .iter()
                .map(to_json_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        other => bail!("unsupported gitdb/toasty value conversion: {other:?}"),
    }
}

pub(crate) fn from_json_value(value: JsonValue, ty: &stmt::Type) -> anyhow::Result<Value> {
    match (ty, value) {
        (_, JsonValue::Null) => Ok(Value::Null),
        (stmt::Type::Bool, JsonValue::Bool(v)) => Ok(Value::Bool(v)),
        (stmt::Type::String, JsonValue::String(v)) => Ok(Value::String(v)),
        (stmt::Type::Uuid, JsonValue::String(v)) => v
            .parse()
            .map(Value::Uuid)
            .map_err(|_| anyhow!("invalid gitdb result: invalid uuid: {v}")),
        (stmt::Type::I8, JsonValue::Number(n)) => n
            .as_i64()
            .and_then(|v| i8::try_from(v).ok())
            .map(Value::I8)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected i8")),
        (stmt::Type::I16, JsonValue::Number(n)) => n
            .as_i64()
            .and_then(|v| i16::try_from(v).ok())
            .map(Value::I16)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected i16")),
        (stmt::Type::I32, JsonValue::Number(n)) => n
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .map(Value::I32)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected i32")),
        (stmt::Type::I64, JsonValue::Number(n)) => n
            .as_i64()
            .map(Value::I64)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected i64")),
        (stmt::Type::U8, JsonValue::Number(n)) => n
            .as_u64()
            .and_then(|v| u8::try_from(v).ok())
            .map(Value::U8)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected u8")),
        (stmt::Type::U16, JsonValue::Number(n)) => n
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .map(Value::U16)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected u16")),
        (stmt::Type::U32, JsonValue::Number(n)) => n
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .map(Value::U32)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected u32")),
        (stmt::Type::U64, JsonValue::Number(n)) => n
            .as_u64()
            .map(Value::U64)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected u64")),
        (stmt::Type::F32, JsonValue::Number(n)) => n
            .as_f64()
            .map(|v| Value::F32(v as f32))
            .ok_or_else(|| anyhow!("invalid gitdb result: expected f32")),
        (stmt::Type::F64, JsonValue::Number(n)) => n
            .as_f64()
            .map(Value::F64)
            .ok_or_else(|| anyhow!("invalid gitdb result: expected f64")),
        (stmt::Type::Bytes, JsonValue::Array(items)) => items
            .into_iter()
            .map(|item| match item {
                JsonValue::Number(n) => n
                    .as_u64()
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or_else(|| anyhow!("invalid gitdb result: expected byte")),
                _ => bail!("invalid gitdb result: expected numeric byte array"),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Bytes),
        (stmt::Type::List(elem_ty), JsonValue::Array(items)) => items
            .into_iter()
            .map(|item| from_json_value(item, elem_ty))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
        (stmt::Type::Record(field_tys), JsonValue::Array(items)) => {
            if items.len() != field_tys.len() {
                let message = format!(
                    "record width mismatch: expected {}, got {}",
                    field_tys.len(),
                    items.len()
                );
                anyhow::bail!("invalid gitdb result: {message}");
            }

            items
                .into_iter()
                .zip(field_tys.iter())
                .map(|(item, ty)| from_json_value(item, ty))
                .collect::<Result<Vec<_>, _>>()
                .map(stmt::ValueRecord::from_vec)
                .map(Value::from)
        }
        (ty, value) => bail!("invalid gitdb result: cannot decode {value:?} as {ty:?}"),
    }
}
