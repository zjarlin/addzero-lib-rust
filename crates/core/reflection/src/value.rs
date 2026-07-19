use serde::Serialize;
use serde_json::Value;

pub fn contains_ignore_order(seq: impl AsRef<str>, search_seq: impl AsRef<str>) -> bool {
    let seq = seq.as_ref();
    let search_seq = search_seq.as_ref();
    seq.contains(search_seq) || search_seq.contains(seq)
}

pub fn is_new<T: Serialize>(value: &T) -> bool {
    let Ok(value) = serde_json::to_value(value) else {
        return false;
    };

    match value {
        Value::Object(fields) => fields.values().all(is_emptyish),
        other => is_emptyish(&other),
    }
}

pub fn is_not_new<T: Serialize>(value: &T) -> bool {
    !is_new(value)
}

pub fn is_collection_value(value: &Value) -> bool {
    matches!(value, Value::Array(_))
}

pub fn is_custom_object_value(value: &Value) -> bool {
    matches!(value, Value::Object(_))
}

fn is_emptyish(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(content) => content.trim().is_empty(),
        Value::Array(values) => values.is_empty(),
        _ => false,
    }
}
