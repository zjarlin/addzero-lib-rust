use addzero_browser_automation::{
    BrowserAutomation, BrowserAutomationContextStore, BrowserAutomationOptions, BrowserMode,
    CdpEndpoint, FormFieldDef, normalize_cdp_http_url,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    BrowserAutomationContextStore::set_start_url("www.baidu.com");

    let options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        mode: BrowserMode::Cdp(CdpEndpoint::Http(normalize_cdp_http_url("127.0.0.1:9222"))),
        ..BrowserAutomationOptions::default()
    };

    // 你先在这里“录制”动作：按顺序把 selector 和 value 填进去。
    let fields = vec![
        FormFieldDef::input(
            "keyword",
            ["input[name='wd']", "input#kw", "input.s_ipt"],
            "cloudflare cdp rust",
        )
        .required(true),
        FormFieldDef::click("search", ["input#su", "button[type='submit']"]).required(true),
    ];

    BrowserAutomation::fill_from_context(&fields, &options, None)?;
    Ok(())
}
