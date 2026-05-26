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

// test
