//! 调试：打开 OpenAI create-account 页面，打印所有 visible input 的详细信息。
//!
//! ```bash
//! cargo run -p az-browser-automation --example debug_page
//! ```

use az_browser_automation::ai_reg_auto::openai::*;
use az_browser_automation::{BrowserAutomation, BrowserAutomationOptions};
use headless_chrome::protocol::cdp::Runtime;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let options = BrowserAutomationOptions {
        debug: true,
        headless: false,
        timeout_ms: 30000,
        ..Default::default()
    };

    BrowserAutomation::with_tab(OpenAiAuthOptions::SIGN_UP_URL, &options, |tab| {
        std::thread::sleep(Duration::from_secs(3));

        let result = tab.call_method(Runtime::Evaluate {
                expression: r#"
                (() => {
                    const visible = (el) => {
                        const style = window.getComputedStyle(el);
                        const rect = el.getBoundingClientRect();
                        return !el.disabled
                            && style.visibility !== 'hidden'
                            && style.display !== 'none'
                            && rect.width > 0 && rect.height > 0;
                    };
                    const inputs = [...document.querySelectorAll('input, textarea, button, a[role="button"]')]
                        .filter(visible)
                        .map(el => ({
                            tag: el.tagName,
                            type: el.type || '',
                            name: el.name || '',
                            id: el.id || '',
                            autocomplete: el.autocomplete || '',
                            placeholder: el.placeholder || '',
                            ariaLabel: el.getAttribute('aria-label') || '',
                            dataTestid: el.getAttribute('data-testid') || '',
                            className: el.className || '',
                            text: (el.innerText || el.textContent || el.value || '').slice(0, 80),
                            rect: {
                                x: el.getBoundingClientRect().x | 0,
                                y: el.getBoundingClientRect().y | 0,
                                w: el.getBoundingClientRect().width | 0,
                                h: el.getBoundingClientRect().height | 0,
                            }
                        }));
                    return JSON.stringify({
                        url: window.location.href,
                        title: document.title,
                        bodySnippet: (document.body?.innerText || '').slice(0, 500),
                        count: inputs.length,
                        inputs: inputs,
                    }, null, 2);
                })()
                "#.to_owned(),
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
            }).map_err(|e| az_browser_automation::BrowserAutomationError::Browser(e.to_string()))?;

        if let Some(val) = result.result.value {
            if let Some(s) = val.as_str() {
                println!("{}", s);
            }
        }

        Ok(())
    })?;

    Ok(())
}
