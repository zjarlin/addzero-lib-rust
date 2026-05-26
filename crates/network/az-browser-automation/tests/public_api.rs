use az_browser_automation::browser_automation::*;

#[test]
fn context_store_round_trip_keeps_start_url() {
    BrowserAutomationContextStore::clear();
    BrowserAutomationContextStore::set_start_url("www.baidu.com");

    // Bare hosts must be normalized so downstream browser navigation gets a valid URL.
    assert_eq!(
        BrowserAutomationContextStore::start_url().as_deref(),
        Some("https://www.baidu.com")
    );

    BrowserAutomationContextStore::clear();
    // Clearing the global store must isolate later automation runs and tests.
    assert!(BrowserAutomationContextStore::get().is_none());
}

#[test]
fn field_type_all_lists_supported_form_actions() {
    assert_eq!(
        FieldType::ALL,
        &[FieldType::Input, FieldType::Click, FieldType::Check]
    );
}

#[test]
fn field_type_code_returns_snake_case_wire_code() {
    assert_eq!(FieldType::Input.code(), "input");
}

#[test]
fn field_type_from_code_parses_supported_wire_code() {
    assert_eq!(FieldType::from_code("click"), Some(FieldType::Click));
}

#[test]
fn field_type_serde_uses_snake_case_wire_code() {
    let serialized = serde_json::to_string(&FieldType::Check).expect("serialize field type");

    assert_eq!(serialized, "\"check\"");
}

#[test]
fn field_type_serde_reads_snake_case_wire_code() {
    let field_type: FieldType = serde_json::from_str("\"click\"").expect("deserialize field type");

    assert_eq!(field_type, FieldType::Click);
}
