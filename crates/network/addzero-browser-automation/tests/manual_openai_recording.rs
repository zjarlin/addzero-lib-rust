use addzero_browser_automation::{
    BrowserAutomation, BrowserAutomationContextStore, BrowserAutomationOptions, BrowserMode,
    CdpEndpoint, normalize_cdp_http_url,
};
use std::time::Duration;

#[test]
#[ignore = "manual recording test: requires a running CDP Chrome session and human interaction"]
fn openai_login_page_opens_for_manual_recording() {
    BrowserAutomationContextStore::clear();
    BrowserAutomationContextStore::set_start_url("https://auth.openai.com/log-in");

    let options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        timeout_ms: 120_000,
        mode: BrowserMode::Cdp(CdpEndpoint::Http(normalize_cdp_http_url("127.0.0.1:9222"))),
        ..BrowserAutomationOptions::default()
    };

    BrowserAutomation::open_and_hold_from_context(&options, Duration::from_secs(600))
        .expect("should open OpenAI login page and keep the tab available for recording");
}
