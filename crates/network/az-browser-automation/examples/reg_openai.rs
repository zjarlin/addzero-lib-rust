//! OpenAI browser-based full registration using CDP mode.
//!
//! 1. 手动启动 Chromium.app:
//!    `/Applications/Chromium.app/Contents/MacOS/Chromium \
//!        --remote-debugging-port=9222 \
//!        --user-data-dir=/tmp/chromium-debug \
//!        --no-first-run --no-sandbox`
//!
//! 2. 运行注册程序:
//!    `cargo run -p az-browser-automation --example reg_openai`
//!
//! 手动处理 CF 人机校验后，程序会继续执行。

use az_browser_automation::ai_reg_auto::openai::*;
use az_browser_automation::proxy::ProxyConfig;
use az_browser_automation::{
    BrowserAutomationContextStore, BrowserAutomationOptions, BrowserMode, CdpEndpoint,
    OpenAiAuthAutomation, normalize_cdp_http_url,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedEmail {
    email: String,
    email_password: String,
    jwt: String,
    openai_password: String,
    stage: String,
    created_at: String,
}

fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex_auto_reg")
        .join("unuse_email")
}

fn load_cached() -> Option<CachedEmail> {
    let dir = cache_dir();
    for entry in fs::read_dir(&dir).ok()? {
        let path = entry.ok()?.path();
        if path.extension().map_or(false, |e| e == "json") {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(c) = serde_json::from_str::<CachedEmail>(&data) {
                    let _ = fs::remove_file(&path);
                    return Some(c);
                }
            }
        }
    }
    None
}

fn save_cached(c: &CachedEmail) {
    let dir = cache_dir();
    fs::create_dir_all(&dir).ok();
    let name = c.email.replace('@', "_at_");
    fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_string_pretty(c).unwrap_or_default(),
    )
    .ok();
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let d = secs / 86400;
    let r = secs % 86400;
    let year = 1970 + d / 365;
    let doy = d % 365;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        year,
        1 + (doy / 30).min(11),
        1 + (doy % 30),
        r / 3600,
        (r % 3600) / 60,
        r % 60,
    )
}

fn random_email_prefix() -> String {
    let first = [
        "james", "mary", "robert", "patricia", "john", "jennifer", "michael", "linda", "david",
        "elizabeth", "william", "susan", "richard", "jessica", "joseph", "sarah", "thomas",
        "karen", "charles", "nancy", "daniel", "lisa", "matthew", "betty", "anthony", "margaret",
        "donald", "sandra", "steven", "ashley", "paul", "kimberly", "andrew", "emily", "joshua",
        "donna", "kenneth", "michelle", "kevin", "carol", "brian", "amanda", "george", "melissa",
        "timothy", "deborah", "ronald", "stephanie",
    ];
    let last = [
        "smith", "johnson", "williams", "brown", "jones", "garcia", "miller", "davis",
        "rodriguez", "martinez", "hernandez", "lopez", "gonzalez", "wilson", "anderson", "thomas",
        "taylor", "moore", "jackson", "martin", "lee", "perez", "thompson", "white", "harris",
        "sanchez", "clark", "ramirez", "lewis", "robinson", "walker", "young", "allen", "king",
        "wright", "scott", "torres", "nguyen",
    ];
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let fi = ((seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407))
        >> 33) as usize
        % first.len();
    let li = ((seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407))
        >> 33) as usize
        % last.len();
    format!("{}{}", first[fi], last[li])
}

fn main() -> Result<(), Box<dyn Error>> {
    // CDP 模式连接到已启动的 Chromium
    let cdp_port = env::var("CDP_PORT").unwrap_or_else(|_| "9222".to_owned());
    let cdp_endpoint = CdpEndpoint::Http(normalize_cdp_http_url(&cdp_port));

    let browser_options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        timeout_ms: 120_000,
        mode: BrowserMode::Cdp(cdp_endpoint.clone()),
        ..BrowserAutomationOptions::default()
    };

    // Optional proxy — set BROWSER_PROXY env var
    if let Ok(proxy_url) = env::var("BROWSER_PROXY") {
        if let Ok(_proxy) = ProxyConfig::from_url(&proxy_url) {
            // proxy 通过 Chromium 启动参数注入，这里仅记录
            println!("代理: {proxy_url}");
        }
    }

    println!("=== OpenAI 注册 (CDP 模式) ===");
    println!("CDP 端口: {cdp_port}");
    println!("headless: {}", browser_options.headless);
    println!();

    let sms_token = env::var("FIVESIM_TOKEN").ok();

    // Reuse cached email if available
    let cached = load_cached();
    let (prefix, reuse_password) = if let Some(ref c) = cached {
        println!("♻ 复用: {}", c.email);
        (
            c.email.split('@').next().unwrap_or("reuse").to_string(),
            Some(c.openai_password.clone()),
        )
    } else {
        let p = random_email_prefix();
        println!("🆕 新建: {p}");
        (p, None)
    };

    let reg_options = OpenAiFullRegOptions {
        start_url: OpenAiAuthOptions::SIGN_UP_URL.to_owned(),
        password: reuse_password,
        sms_token,
        sms_product: "openai".to_owned(),
        sms_country: "usa".to_owned(),
        sms_operator: "any".to_owned(),
        email_prefix: prefix,
        step_delay: Duration::from_millis(800),
        hold_for: Some(Duration::from_secs(30)),
    };

    BrowserAutomationContextStore::clear();
    BrowserAutomationContextStore::set_start_url(&reg_options.start_url);

    let result = OpenAiRegAutomation::run_full_registration(&reg_options, &browser_options)?;

    // Cache if stuck on about-you (account usable, onboarding incomplete)
    if result.final_url.contains("about-you") {
        save_cached(&CachedEmail {
            email: result.email.clone(),
            email_password: result.email_password.clone(),
            jwt: result.jwt_token.clone(),
            openai_password: result.openai_password.clone(),
            stage: "about-you".into(),
            created_at: now_iso(),
        });
        println!("📦 已缓存 ~/.codex_auto_reg/unuse_email/");
    }

    println!("\n=== 结果 ===");
    println!("邮件:     {}", result.email);
    println!("邮箱密码: {}", result.email_password);
    println!("JWT:      {}", result.jwt_token);
    println!("OA密码:   {}", result.openai_password);
    println!("阶段:     {:?}", result.stage);
    println!("最终URL:  {}", result.final_url);
    println!("消息:     {}", result.message);

    Ok(())
}
