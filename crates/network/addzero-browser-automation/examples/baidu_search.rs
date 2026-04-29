use addzero_browser_automation::{
    BrowserAutomation, BrowserAutomationContextStore, BrowserAutomationOptions, BrowserMode,
    CdpEndpoint, normalize_cdp_http_url,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BrowserAutomationContextStore::set_baidu();

    let options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        mode: BrowserMode::Cdp(CdpEndpoint::Http(normalize_cdp_http_url("127.0.0.1:9222"))),
        ..BrowserAutomationOptions::default()
    };

    BrowserAutomation::run_baidu_search("Rust 浏览器自动化", &options)?;
    Ok(())
}
