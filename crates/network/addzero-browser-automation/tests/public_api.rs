use addzero_browser_automation::*;

#[test]
fn debug_mode_forces_headful_browser() {
    let options = BrowserAutomationOptions {
        debug: true,
        headless: true,
        ..BrowserAutomationOptions::default()
    };

    assert!(!options.effective_headless());
}

#[test]
fn context_store_round_trip_keeps_start_url() {
    BrowserAutomationContextStore::clear();
    BrowserAutomationContextStore::set_start_url("www.baidu.com");

    assert_eq!(
        BrowserAutomationContextStore::start_url().as_deref(),
        Some("https://www.baidu.com")
    );

    BrowserAutomationContextStore::clear();
    assert!(BrowserAutomationContextStore::get().is_none());
}

#[test]
fn form_field_builders_default_to_expected_field_types() {
    let input = FormFieldDef::input("keyword", ["input[name='wd']"], "rust").required(true);
    let click = FormFieldDef::click("search", ["#su"]);
    let check = FormFieldDef::check("remember", ["#remember"]);

    assert_eq!(input.field_type, FieldType::Input);
    assert!(input.required);
    assert_eq!(click.field_type, FieldType::Click);
    assert_eq!(check.field_type, FieldType::Check);
}

#[test]
fn browser_defaults_to_cdp_mode() {
    let options = BrowserAutomationOptions::default();

    assert_eq!(
        options.mode,
        BrowserMode::Cdp(CdpEndpoint::Http("http://127.0.0.1:9222".to_owned()))
    );
}

#[test]
fn normalize_cdp_http_url_adds_scheme_and_trims_slash() {
    assert_eq!(
        normalize_cdp_http_url("127.0.0.1:9222/"),
        "http://127.0.0.1:9222"
    );
    assert_eq!(
        normalize_cdp_http_url("http://localhost:9333/"),
        "http://localhost:9333"
    );
}

#[test]
fn cdp_port_parser_extracts_port_from_http_url() {
    assert_eq!(parse_cdp_port("http://127.0.0.1:9222"), Some(9222));
    assert_eq!(parse_cdp_port("http://localhost:9333/"), Some(9333));
}
