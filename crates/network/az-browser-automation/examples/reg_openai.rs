//! OpenAI registration using a real QQ email (no disposable mail).
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
//! 程序会在需要邮箱验证码时提示手动输入。

use az_browser_automation::ai_reg_auto::openai::*;
use az_browser_automation::{
    BrowserAutomation, BrowserAutomationError, BrowserAutomationOptions,
    BrowserMode, CdpEndpoint, normalize_cdp_http_url,
};
use az_sms::SmsProvider;
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════
// JS helpers — mirror the internal helpers in openai.rs
// ═══════════════════════════════════════════════════════════════════════

fn eval(tab: &Arc<Tab>, js: &str) -> Result<Value, BrowserAutomationError> {
    let result = tab
        .call_method(Runtime::Evaluate {
            expression: js.to_owned(),
            object_group: None,
            include_command_line_api: Some(false),
            silent: Some(false),
            context_id: None,
            return_by_value: Some(true),
            generate_preview: Some(false),
            user_gesture: Some(true),
            await_promise: Some(true),
            throw_on_side_effect: None,
            timeout: None,
            disable_breaks: None,
            repl_mode: None,
            allow_unsafe_eval_blocked_by_csp: None,
            unique_context_id: None,
            serialization_options: None,
        })
        .map_err(|e| BrowserAutomationError::Browser(e.to_string()))?;
    if let Some(ex) = result.exception_details {
        return Err(BrowserAutomationError::Browser(format!("{ex:?}")));
    }
    Ok(result.result.value.unwrap_or(Value::Null))
}

fn eval_bool(tab: &Arc<Tab>, js: &str) -> Result<bool, BrowserAutomationError> {
    eval(tab, js).map(|v| v.as_bool().unwrap_or(false))
}

fn eval_str(tab: &Arc<Tab>, js: &str) -> Result<String, BrowserAutomationError> {
    eval(tab, js).map(|v| v.as_str().unwrap_or("").to_string())
}

/// Read current page state (mirrors `read_state` in openai.rs).
fn page_state(tab: &Arc<Tab>) -> Result<AuthPageState, BrowserAutomationError> {
    let value = eval(tab, r#"
        (() => {
            const visible = (el) => {
                const style = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                return style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0 && rect.height > 0;
            };
            const inputs = [...document.querySelectorAll('input, textarea')].filter(visible);
            const bodyText = document.body ? document.body.innerText.toLowerCase() : '';
            const hasPwd = inputs.some(el => ['type','name','id','autocomplete','placeholder','aria-label']
                .map(a => (el[a]||'').toLowerCase()).join(' ').includes('password'));
            const href = window.location.href.toLowerCase();
            const hasOnboarding = href.includes('about-you') || href.includes('onboarding')
                || bodyText.includes('about you') || bodyText.includes('完成帐户') || bodyText.includes('完成账户');
            const hasVerify = href.includes('verify') || href.includes('verification') || href.includes('code')
                || ['verify','verification','check your email','security code','multi-factor','two-factor',
                    'authenticator','one-time code','验证','验证码','邮箱验证','安全码']
                   .some(t => bodyText.includes(t));
            const hasRejected = bodyText.includes('使用条款') || bodyText.includes('terms of use')
                || bodyText.includes('terms of service') || bodyText.includes('cannot create')
                || bodyText.includes('无法创建');
            const hasCaptcha = bodyText.includes('captcha')
                || document.querySelector('[class*="captcha" i], [id*="captcha" i], iframe[src*="captcha" i], iframe[src*="hcaptcha" i], iframe[src*="recaptcha" i], iframe[src*="turnstile" i]') !== null;
            return { url: window.location.href, title: document.title,
                hasPasswordInput: hasPwd, hasVerification: hasVerify, hasCaptcha: hasCaptcha,
                hasOnboarding: hasOnboarding, hasTermsRejected: hasRejected };
        })()
    "#)?;
    serde_json::from_value(value)
        .map_err(|e| BrowserAutomationError::Browser(e.to_string()))
}

/// Click a button whose text matches one of the given labels.
fn click_button(tab: &Arc<Tab>, labels: &[&str]) -> Result<bool, BrowserAutomationError> {
    let labels_json = serde_json::to_string(labels).unwrap();
    let js = format!(
        r#"(() => {{
            const labels = {labels_json}.map(l => l.toLowerCase());
            const visible = el => {{ const s = window.getComputedStyle(el); const r = el.getBoundingClientRect();
                return !el.disabled && s.visibility!=='hidden' && s.display!=='none' && r.width>0 && r.height>0; }};
            const nodes = [...document.querySelectorAll('button, a, [role="button"], input[type="button"], input[type="submit"]')];
            const tgt = nodes.find(el => visible(el) && labels.some(l => [el.innerText,el.textContent,el.value,el.getAttribute('aria-label')].join(' ').trim().toLowerCase().includes(l)));
            if (!tgt) return false;
            tgt.click(); return true;
        }})()"#
    );
    eval_bool(tab, &js)
}

/// Click the first visible element matching any of the CSS selectors.
fn click_first_visible(tab: &Arc<Tab>, selectors: &[&str]) -> Result<bool, BrowserAutomationError> {
    let sels = serde_json::to_string(selectors).unwrap();
    let js = format!(
        r#"(() => {{
            const selectors = {sels};
            const vis = el => {{ const s=window.getComputedStyle(el); const r=el.getBoundingClientRect();
                return !el.disabled && s.visibility!=='hidden' && s.display!=='none' && r.width>0 && r.height>0; }};
            for (const sel of selectors) {{
                const tgt = [...document.querySelectorAll(sel)].find(vis);
                if (tgt) {{ tgt.click(); return true; }}
            }}
            return false;
        }})()"#
    );
    eval_bool(tab, &js)
}

/// Fill the first visible input matching selectors.
fn fill_input(tab: &Arc<Tab>, selectors: &[&str], value: &str) -> Result<bool, BrowserAutomationError> {
    let sels = serde_json::to_string(selectors).unwrap();
    let val = serde_json::to_string(value).unwrap();
    let js = format!(
        r#"(() => {{
            const selectors = {sels}; const value = {val};
            const vis = el => {{ const s=window.getComputedStyle(el); const r=el.getBoundingClientRect();
                return !el.disabled && !el.readOnly && s.visibility!=='hidden' && s.display!=='none' && r.width>0 && r.height>0; }};
            for (const sel of selectors) {{
                for (const el of document.querySelectorAll(sel)) {{
                    if (!vis(el) || !('value' in el)) continue;
                    el.focus();
                    const d = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value');
                    if (d && d.set) d.set.call(el, value); else el.value = value;
                    el.dispatchEvent(new InputEvent('input', {{ bubbles:true, data:value, inputType:'insertText' }}));
                    el.dispatchEvent(new Event('change', {{ bubbles:true }}));
                    return true;
                }}
            }}
            return false;
        }})()"#
    );
    eval_bool(tab, &js)
}

/// Fill verification code input.
fn fill_code(tab: &Arc<Tab>, code: &str) -> Result<bool, BrowserAutomationError> {
    fill_input(
        tab,
        &[
            "input[type='text'][maxlength='6']",
            "input[inputmode='numeric']",
            "input[name*='code' i]",
            "input[name*='otp' i]",
            "input[id*='code' i]",
            "input[id*='otp' i]",
            "input[placeholder*='code' i]",
            "input[placeholder*='6-digit' i]",
            "input[aria-label*='code' i]",
        ],
        code,
    )
}

/// Fill password input.
fn fill_password(tab: &Arc<Tab>, password: &str) -> Result<bool, BrowserAutomationError> {
    fill_input(
        tab,
        &[
            "input[type='password']",
            "input[name='password']",
            "input[autocomplete='current-password']",
            "input[autocomplete='new-password']",
            "input[id*='password' i]",
            "input[placeholder*='password' i]",
        ],
        password,
    )
}

/// Fill all visible textboxes (for onboarding: name + age).
fn fill_textboxes(tab: &Arc<Tab>, values: &[&str]) -> Result<bool, BrowserAutomationError> {
    let vals_json = serde_json::to_string(values).unwrap();
    let js = format!(
        r#"(() => {{
            const values = {vals_json};
            const boxes = [...document.querySelectorAll('[role="textbox"]'),
                ...document.querySelectorAll('input[type="text"]:not([readonly]):not([disabled])'),
                ...document.querySelectorAll('input:not([type]):not([readonly]):not([disabled])'),
                ...document.querySelectorAll('input[type="number"]:not([readonly]):not([disabled])')];
            const vis = el => {{ const s=window.getComputedStyle(el); const r=el.getBoundingClientRect();
                return !el.readOnly && !el.disabled && s.visibility!=='hidden' && s.display!=='none' && r.width>0 && r.height>0; }};
            let filled = 0;
            for (const el of boxes) {{
                if (!vis(el) || filled >= values.length) continue;
                const value = values[filled];
                el.focus();
                if (document.activeElement === el) {{ document.execCommand('selectAll', false, null);
                    document.execCommand('insertText', false, value); }}
                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                    const ns = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value');
                    if (ns && ns.set) ns.set.call(el, value); else el.value = value;
                }} else {{ el.textContent = value; }}
                el.dispatchEvent(new InputEvent('input', {{ bubbles:true, data:value, inputType:'insertText' }}));
                el.dispatchEvent(new Event('change', {{ bubbles:true }}));
                filled++;
            }}
            return filled === values.length;
        }})()"#
    );
    eval_bool(tab, &js)
}

/// Fill phone number input.
fn fill_phone(tab: &Arc<Tab>, phone: &str) -> Result<bool, BrowserAutomationError> {
    fill_input(
        tab,
        &[
            "input[type='tel']",
            "input[name*='phone' i]",
            "input[name*='mobile' i]",
            "input[id*='phone' i]",
            "input[id*='mobile' i]",
            "input[autocomplete='tel']",
            "input[autocomplete='tel-national']",
            "input[placeholder*='phone' i]",
            "input[aria-label*='phone' i]",
        ],
        phone,
    )
}

/// Wait for a page state condition.
fn wait_state(
    tab: &Arc<Tab>,
    timeout: Duration,
    accept: impl Fn(&AuthPageState) -> bool,
) -> Result<AuthPageState, BrowserAutomationError> {
    let deadline = Instant::now() + timeout;
    let mut last = page_state(tab)?;
    while Instant::now() < deadline {
        if accept(&last) {
            return Ok(last);
        }
        thread::sleep(Duration::from_millis(500));
        last = page_state(tab)?;
    }
    Ok(last)
}

fn random_jitter() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
    let secs = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as u64 % 60 + 3;
    thread::sleep(Duration::from_secs(secs));
}

fn random_string(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$";
    let mut state = seed as u64;
    (0..len)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            chars[(state >> 33) as usize % chars.len()] as char
        })
        .collect()
}

fn random_full_name() -> String {
    let first = ["James","Mary","Robert","Patricia","John","Jennifer","Michael","Linda",
        "David","Elizabeth","William","Barbara","Richard","Susan","Joseph","Jessica",
        "Thomas","Sarah","Christopher","Karen","Charles","Lisa","Daniel","Nancy"];
    let last = ["Smith","Johnson","Williams","Brown","Jones","Garcia","Miller","Davis",
        "Rodriguez","Martinez","Hernandez","Lopez","Gonzalez","Wilson","Anderson",
        "Thomas","Taylor","Moore","Jackson","Martin","Lee","Perez","Thompson"];
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
    let fi = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as usize % first.len();
    let li = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as usize % last.len();
    format!("{} {}", first[fi], last[li])
}

fn prompt(msg: &str) -> String {
    print!("{msg}");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().to_string()
}

// ═══════════════════════════════════════════════════════════════════════
// Auth page state (mirrors AuthPageState in openai.rs)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthPageState {
    url: String,
    title: String,
    #[allow(unused)]
    has_password_input: bool,
    has_verification: bool,
    has_captcha: bool,
    has_onboarding: bool,
    has_terms_rejected: bool,
}

impl AuthPageState {
    fn is_authenticated(&self) -> bool {
        let url = self.url.to_ascii_lowercase();
        // Exclude auth/login URLs that happen to be on chatgpt.com
        let is_auth_page = url.contains("/auth/") || url.contains("/login");
        !is_auth_page
            && (url.contains("platform.openai.com")
                || url.contains("chatgpt.com")
                || url.contains("chat.openai.com"))
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════

fn main() -> Result<(), Box<dyn Error>> {
    let cdp_port = env::var("CDP_PORT").unwrap_or_else(|_| "127.0.0.1:9222".to_owned());
    let cdp_endpoint = CdpEndpoint::Http(normalize_cdp_http_url(&cdp_port));

    let browser_options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        slow_mo_ms: 300,
        timeout_ms: 120_000,
        mode: BrowserMode::Cdp(cdp_endpoint.clone()),
        ..BrowserAutomationOptions::default()
    };

    if let Ok(proxy_url) = env::var("BROWSER_PROXY") {
        println!("代理: {proxy_url}");
    }

    // ── 硬编码 QQ 邮箱，排除临时邮箱黑名单 ──
    let email = "1595656317@qq.com".to_string();
    let password = random_string(16);
    let sms_token = env::var("FIVESIM_TOKEN").ok();

    println!("=== OpenAI 注册 (QQ邮箱) ===");
    println!("邮箱:   {email}");
    println!("密码:   {password}");
    println!("CDP端口: {cdp_port}");
    println!();

    BrowserAutomation::with_tab(OpenAiAuthOptions::SIGN_UP_URL, &browser_options, |tab| {
        thread::sleep(Duration::from_millis(1500));
        random_jitter();

        // ── Dismiss stale session ──
        let body = eval_str(tab, "document.body ? document.body.innerText.slice(0,500) : ''")?;
        if body.to_lowercase().contains("session ended")
            || body.contains("会话已结束")
        {
            println!("[关闭会话过期提示]");
            let _ = click_button(tab, &["Log in", "Login", "登录"]);
            thread::sleep(Duration::from_millis(1500));
        }

        // ── Click "Sign up" if on login page ──
        let _ = click_button(tab, &["Sign up", "Create account", "注册", "Get started"]);
        thread::sleep(Duration::from_millis(1500));

        let mut state = page_state(tab)?;
        println!("[初始] url={}", state.url);

        if state.is_authenticated() {
            println!("✅ 已登录，无需注册");
            return Ok(());
        }

        // ── Fill email ──
        // Diagnostic: dump page state
        let body_snippet = eval(tab, "document.body ? document.body.innerText.slice(0,800) : 'no body'")?;
        println!("[诊断] body={:?}", body_snippet);
        let input_count = eval(tab, "document.querySelectorAll('input').length")?;
        println!("[诊断] input_count={:?}", input_count);

        let email_selectors: &[&str] = &[
            "input[type='email']",
            "input[name='email']",
            "input[name='username']",
            "input[autocomplete='email']",
            "input[autocomplete='username']",
            "input[id*='email' i]",
            "input[id*='username' i]",
            "input[placeholder*='email' i]",
        ];
        if !fill_input(tab, email_selectors, &email)? {
            println!("❌ 未找到邮箱输入框");
            return Ok(());
        }
        println!("[填入邮箱] {email}");

        // Submit the form containing the email input (avoids misclicking Google/Apple buttons)
        let submitted = eval_bool(tab, r#"
            (() => {
                const inputs = [...document.querySelectorAll('input')];
                const emailInput = inputs.find(el => {
                    const hay = [el.type, el.name, el.id, el.autocomplete, el.placeholder].join(' ').toLowerCase();
                    return hay.includes('email') && el.offsetParent !== null;
                });
                if (!emailInput) return false;
                const form = emailInput.closest('form');
                if (form) { form.dispatchEvent(new Event('submit', {bubbles:true, cancelable:true})); return true; }
                // No form wrapper — try Enter key on the input
                emailInput.dispatchEvent(new KeyboardEvent('keydown', {bubbles:true, key:'Enter', code:'Enter', keyCode:13}));
                emailInput.dispatchEvent(new KeyboardEvent('keypress', {bubbles:true, key:'Enter', code:'Enter', keyCode:13}));
                return true;
            })()
        "#)?;
        if !submitted {
            // Fallback to button click
            if !click_button(tab, &["Continue", "Next", "继续"])? {
                println!("❌ 未找到提交按钮");
                return Ok(());
            }
        }
        println!("[提交邮箱]");

        random_jitter();

        state = wait_state(tab, Duration::from_secs(30), |s| {
            s.is_authenticated() || s.has_password_input || s.has_verification || s.has_captcha || s.has_terms_rejected
        })?;
        println!("[邮箱提交后] url={} verify={} pwd={} captcha={} rejected={}",
            state.url, state.has_verification, state.has_password_input, state.has_captcha, state.has_terms_rejected);

        if state.has_terms_rejected {
            println!("❌ 被风控拒绝（使用条款）");
            return Ok(());
        }
        if state.has_captcha {
            println!("⚠️ 出现验证码，请手动处理后在浏览器中继续操作");
            println!("按 Enter 键继续...");
            prompt("");
            state = page_state(tab)?;
        }
        if state.is_authenticated() {
            println!("✅ 注册成功！");
            return Ok(());
        }

        // ── Email verification code ──
        if state.has_verification && !state.has_password_input {
            let code = prompt("📧 请输入QQ邮箱中的验证码: ");
            if code.is_empty() {
                println!("❌ 未输入验证码，退出");
                return Ok(());
            }
            fill_code(tab, &code)?;
            println!("[填入验证码] {code}");

            if !click_button(tab, &["Continue", "Verify", "Next", "继续", "验证"])? {
                thread::sleep(Duration::from_secs(3));
            }
            println!("[提交验证码]");

            state = wait_state(tab, Duration::from_secs(25), |s| {
                s.is_authenticated() || s.has_password_input || s.has_captcha || s.has_terms_rejected
            })?;
            println!("[验证码后] url={} pwd={} rejected={}",
                state.url, state.has_password_input, state.has_terms_rejected);

            if state.has_terms_rejected {
                println!("❌ 被风控拒绝（验证码阶段）");
                return Ok(());
            }
        }

        // ── Fill password ──
        if state.has_password_input {
            fill_password(tab, &password)?;
            println!("[填入密码]");

            if !click_button(tab, &["Continue", "Next", "Create account", "继续"])? {
                println!("❌ 未找到密码提交按钮");
                return Ok(());
            }
            println!("[提交密码]");

            random_jitter();

            state = wait_state(tab, Duration::from_secs(30), |s| {
                s.is_authenticated() || s.has_verification || s.has_captcha || s.has_onboarding || s.has_terms_rejected
            })?;
            println!("[密码后] url={} onboard={} verify={} rejected={}",
                state.url, state.has_onboarding, state.has_verification, state.has_terms_rejected);

            if state.has_terms_rejected {
                println!("❌ 被风控拒绝（密码阶段）");
                return Ok(());
            }
            if state.is_authenticated() {
                println!("✅ 注册成功！");
                return Ok(());
            }
        }

        // ── Onboarding (About You: name + age) ──
        if state.has_onboarding {
            random_jitter();
            let name = random_full_name();
            let age = (22 + (random_string(1).as_bytes()[0] as usize % 20)).to_string();
            println!("[onboarding] name={name} age={age}");
            fill_textboxes(tab, &[&name, &age])?;
            thread::sleep(Duration::from_secs(2));

            let _ = click_button(tab, &["Complete account creation", "Create account", "Finish",
                "Continue", "Next", "Submit", "完成帐户创建", "完成"]);
            thread::sleep(Duration::from_secs(3));

            state = wait_state(tab, Duration::from_secs(20), |s| {
                s.is_authenticated() || s.has_captcha || s.has_terms_rejected
            })?;
            println!("[onboarding后] url={} authenticated={} rejected={}",
                state.url, state.is_authenticated(), state.has_terms_rejected);

            if state.has_terms_rejected {
                println!("❌ 被风控拒绝（onboarding阶段）");
                return Ok(());
            }
            if state.is_authenticated() {
                println!("✅ 注册成功！");
                return Ok(());
            }
        }

        // ── Phone verification ──
        if state.has_verification {
            if let Some(ref token) = sms_token {
                // Buy SMS number via 5sim
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| BrowserAutomationError::Browser(e.to_string()))?;
                let (sms_phone, order_id): (String, u64) = rt.block_on(async {
                    let client = az_sms::FivesimClient::from_token(token)
                        .map_err(|e| BrowserAutomationError::Browser(e.to_string()))?;
                    let request = az_sms::SmsActivationRequest::new("usa", "any", "openai")
                        .map_err(|e| BrowserAutomationError::Browser(e.to_string()))?;
                    let order: az_sms::SmsOrder = client.buy_activation_number(request).await
                        .map_err(|e| BrowserAutomationError::Browser(e.to_string()))?;
                    Ok::<_, BrowserAutomationError>((order.phone, order.id))
                })?;
                println!("[购买号码] phone={sms_phone} order={order_id}");

                fill_phone(tab, &sms_phone)?;
                thread::sleep(Duration::from_millis(800));
                let _ = click_button(tab, &["Continue", "Next", "Send code", "继续"]);
                thread::sleep(Duration::from_secs(3));

                // Poll 5sim for SMS code
                let code: Option<String> = rt.block_on(async {
                    let client = az_sms::FivesimClient::from_token(token).ok()?;
                    let options = az_sms::WaitForSmsOptions::new(
                        Duration::from_secs(180), Duration::from_secs(5)).ok()?;
                    match client.wait_for_sms(order_id, options).await {
                        Ok(order) => {
                            if let Some(code) = order.sms.first().and_then(|m| m.code.clone()) {
                                return Some(code);
                            }
                            order.sms.first()
                                .and_then(|m| {
                                    let re = regex::Regex::new(r"\b(\d{4,8})\b").ok()?;
                                    re.captures(&m.text).map(|c| c[1].to_owned())
                                })
                        }
                        Err(_) => None,
                    }
                });

                if let Some(ref code) = code {
                    fill_code(tab, code)?;
                    thread::sleep(Duration::from_millis(500));
                    let _ = click_button(tab, &["Continue", "Verify", "Next", "继续"]);
                    println!("[填入短信验证码] {code}");
                    thread::sleep(Duration::from_secs(3));
                }

                state = wait_state(tab, Duration::from_secs(15), |s| {
                    s.is_authenticated() || s.has_captcha
                })?;
            } else {
                println!("⚠️ 需要手机验证但未设置 FIVESIM_TOKEN");
                let code = prompt("📱 请输入手机验证码（或按 Enter 跳过）: ");
                if !code.is_empty() {
                    fill_code(tab, &code)?;
                    let _ = click_button(tab, &["Continue", "Verify", "Next", "继续"]);
                    thread::sleep(Duration::from_secs(3));
                }
            }
        }

        if state.is_authenticated() {
            println!("✅ 注册成功！");
        } else {
            println!("⚠️ 流程结束");
            println!("   最终URL: {}", state.url);
            println!("   title: {}", state.title);
            println!("   has_captcha: {}", state.has_captcha);
            println!("   has_verification: {}", state.has_verification);
            println!("   has_terms_rejected: {}", state.has_terms_rejected);
            println!("   has_onboarding: {}", state.has_onboarding);
        }

        // Hold the tab open for inspection
        thread::sleep(Duration::from_secs(60));
        Ok(())
    })?;

    Ok(())
}
