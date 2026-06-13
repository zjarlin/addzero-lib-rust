use az_json::{flatten, get_bool, get_f64, get_i64, get_string, get_value, merge, pretty};
use serde_json::json;

#[test]
fn get_value_reads_nested_missing_and_root_paths() {
    let data = json!({ "a": { "b": { "c": 42 } } });
    assert_eq!(get_value(&data, "a.b.c"), Some(&json!(42)));
    assert_eq!(get_value(&data, "a.x.y"), None);
    assert_eq!(get_value(&data, ""), Some(&data));
    assert_eq!(get_value(&data, "b"), None);
}

#[test]
fn typed_getters_extract_expected_json_types() {
    let data = json!({
        "name": "Alice",
        "age": 30,
        "count": 42,
        "ratio": 3.5,
        "active": true
    });

    assert_eq!(get_string(&data, "name"), Some("Alice".to_string()));
    assert_eq!(get_string(&data, "age"), None);
    assert_eq!(get_i64(&data, "count"), Some(42));
    assert_eq!(get_i64(&data, "ratio"), None);
    assert_eq!(get_f64(&data, "ratio"), Some(3.5));
    assert_eq!(get_f64(&data, "count"), Some(42.0));
    assert_eq!(get_bool(&data, "active"), Some(true));
    assert_eq!(get_bool(&data, "count"), None);
}

#[test]
fn merge_recurses_objects_and_overwrites_leaves() {
    let mut base = json!({ "a": 1, "b": { "x": 10 } });
    let overlay = json!({ "b": { "y": 20 }, "c": 3 });
    merge(&mut base, &overlay);
    assert_eq!(base, json!({ "a": 1, "b": { "x": 10, "y": 20 }, "c": 3 }));

    let mut base = json!({ "a": 1 });
    let overlay = json!({ "a": 2 });
    merge(&mut base, &overlay);
    assert_eq!(base, json!({ "a": 2 }));
}

#[test]
fn merge_ignores_non_object_overlay() {
    let mut base = json!({ "a": 1 });
    let overlay = json!("not an object");
    merge(&mut base, &overlay);
    assert_eq!(base, json!({ "a": 1 }));
}

#[test]
fn flatten_uses_dot_paths_for_objects_and_array_indexes() {
    let data = json!({
        "a": {
            "b": 1,
            "c": { "d": 2 }
        },
        "e": [10, 20]
    });
    let flat = flatten(&data);
    assert_eq!(flat["a.b"], json!(1));
    assert_eq!(flat["a.c.d"], json!(2));
    assert_eq!(flat["e.0"], json!(10));
    assert_eq!(flat["e.1"], json!(20));
}

#[test]
fn pretty_formats_json_with_indentation() {
    let data = json!({ "a": 1, "b": [1, 2, 3] });
    let output = pretty(&data);
    assert!(output.contains('\n'));
    assert!(output.contains("\"a\""));
}
