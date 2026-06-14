//! 为 `serde_json` 扩展实用 JSON 工具函数的便捷辅助库。
//!
//! 为 [`serde_json::Value`] 提供点路径查询、类型安全提取、深度合并、
//! 展平以及美化打印等功能。

use std::collections::HashMap;

use serde_json::Value;

/// 通过点分隔路径读取 JSON 值。
///
/// 给定 `"a.b.c"` 这样的路径时，会逐层遍历嵌套对象，并在路径存在时返回对应值。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::get_value;
///
/// let data = json!({ "a": { "b": { "c": 42 } } });
/// assert_eq!(get_value(&data, "a.b.c"), Some(&json!(42)));
/// assert_eq!(get_value(&data, "a.x"), None);
/// ```
pub fn get_value<'a>(json: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(json);
    }
    let mut current = json;
    for key in path.split('.') {
        match current.get(key) {
            Some(v) => current = v,
            None => return None,
        }
    }
    Some(current)
}

/// 通过点分隔路径读取 `String` 值。
///
/// 当路径不存在，或目标值不是字符串时返回 `None`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::get_string;
///
/// let data = json!({ "name": "Alice" });
/// assert_eq!(get_string(&data, "name"), Some("Alice".to_string()));
/// assert_eq!(get_string(&data, "missing"), None);
/// ```
pub fn get_string(json: &Value, path: &str) -> Option<String> {
    get_value(json, path).and_then(|v| v.as_str().map(String::from))
}

/// 通过点分隔路径读取 `i64` 值。
///
/// 当路径不存在，或目标值不能表示为 `i64` 时返回 `None`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::get_i64;
///
/// let data = json!({ "count": 42 });
/// assert_eq!(get_i64(&data, "count"), Some(42));
/// ```
pub fn get_i64(json: &Value, path: &str) -> Option<i64> {
    get_value(json, path).and_then(|v| v.as_i64())
}

/// 通过点分隔路径读取 `f64` 值。
///
/// 当路径不存在，或目标值不能表示为 `f64` 时返回 `None`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::get_f64;
///
/// let data = json!({ "ratio": 3.5 });
/// assert_eq!(get_f64(&data, "ratio"), Some(3.5));
/// ```
pub fn get_f64(json: &Value, path: &str) -> Option<f64> {
    get_value(json, path).and_then(|v| v.as_f64())
}

/// 通过点分隔路径读取 `bool` 值。
///
/// 当路径不存在，或目标值不是布尔值时返回 `None`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::get_bool;
///
/// let data = json!({ "active": true });
/// assert_eq!(get_bool(&data, "active"), Some(true));
/// ```
pub fn get_bool(json: &Value, path: &str) -> Option<bool> {
    get_value(json, path).and_then(|v| v.as_bool())
}

/// 将 `overlay` 深度合并到 `base`。
///
/// 只有双方同为对象时才会递归合并；`overlay` 中的叶子值和非对象值会覆盖 `base`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::merge;
///
/// let mut base = json!({ "a": 1, "b": { "x": 10 } });
/// let overlay = json!({ "b": { "y": 20 }, "c": 3 });
/// merge(&mut base, &overlay);
/// assert_eq!(base, json!({ "a": 1, "b": { "x": 10, "y": 20 }, "c": 3 }));
/// ```
pub fn merge(base: &mut Value, overlay: &Value) {
    if let (Value::Object(base_map), Value::Object(overlay_map)) = (base, overlay) {
        for (key, value) in overlay_map {
            if let Some(base_val) = base_map.get_mut(key) {
                if base_val.is_object() && value.is_object() {
                    merge(base_val, value);
                } else {
                    base_map.insert(key.clone(), value.clone());
                }
            } else {
                base_map.insert(key.clone(), value.clone());
            }
        }
    }
}

/// 将嵌套 JSON 展平成点分隔 key 的 `HashMap`。
///
/// 对象会递归展开；数组使用数字下标 key，例如 `"items.0"`；非容器值会成为叶子条目。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::flatten;
///
/// let data = json!({ "a": { "b": 1 } });
/// let flat = flatten(&data);
/// assert_eq!(flat["a.b"], json!(1));
/// ```
pub fn flatten(json: &Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    flatten_inner(json, String::new(), &mut map);
    map
}

fn flatten_inner(json: &Value, prefix: String, map: &mut HashMap<String, Value>) {
    match json {
        Value::Object(obj) => {
            for (key, value) in obj {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_inner(value, new_key, map);
            }
        }
        Value::Array(arr) => {
            for (i, value) in arr.iter().enumerate() {
                let new_key = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                flatten_inner(value, new_key, map);
            }
        }
        _ => {
            map.insert(prefix, json.clone());
        }
    }
}

/// 将 JSON 值格式化为带缩进的字符串。
///
/// 对有效的 [`Value`] 正常返回 pretty JSON；如果序列化异常失败，则返回 `"<invalid json>"`。
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use az_json::api::pretty;
///
/// let data = json!({ "a": 1 });
/// let s = pretty(&data);
/// assert!(s.contains('\n'));
/// ```
pub fn pretty(json: &Value) -> String {
    serde_json::to_string_pretty(json).unwrap_or_else(|_| "<invalid json>".to_string())
}
