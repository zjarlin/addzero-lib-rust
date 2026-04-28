use addzero_dict_spec::{DictionarySpec, RawValueKind};

#[test]
fn parses_and_validates_int_dictionary() {
    let spec = DictionarySpec::from_json_str(
        r#"{
          "code":"board_parity",
          "name":"Board Parity",
          "description":"Parity",
          "scope":"board.static",
          "rawValueKind":"int",
          "openEnum":true,
          "unknownVariant":"Other",
          "items":[
            {"code":"NONE","label":"None","rawIntValue":0,"sortIndex":10,"enabled":true},
            {"code":"ODD","label":"Odd","rawIntValue":2,"sortIndex":20,"enabled":true}
          ]
        }"#,
    )
    .expect("spec should parse");

    assert_eq!(spec.code, "board_parity");
    assert_eq!(spec.raw_value_kind, RawValueKind::Int);
    assert!(spec.open_enum);
    assert_eq!(spec.normalized_unknown_variant(), "Other");
    assert_eq!(spec.items.len(), 2);
}

#[test]
fn rejects_duplicate_raw_values() {
    let error = DictionarySpec::from_json_str(
        r#"{
          "code":"board_parity",
          "name":"Board Parity",
          "scope":"board.static",
          "rawValueKind":"int",
          "items":[
            {"code":"NONE","label":"None","rawIntValue":0},
            {"code":"EVEN","label":"Even","rawIntValue":0}
          ]
        }"#,
    )
    .expect_err("duplicate raw values should fail");

    assert!(error.to_string().contains("duplicate rawIntValue"));
}
