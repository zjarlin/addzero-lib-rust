//! Kiro (AWS Builder ID) registration flow.
//!
//! Automates the sign-up process at `https://app.kiro.dev/signin` through the
//! AWS Builder ID provider using a disposable email address from `az-temp-mail`.

use crate::browser_automation::BrowserAutomationResult;
use crate::registration::{RegistrationFlow, RegistrationResult, extract_verification_code};
use crate::session::BrowserSession;
use az_derive_aliases::{apply, plain_default_copy_eq};
use az_temp_mail::{PageRequest, TempMailMailbox, TempMailProvider};
use std::thread;
use std::time::Duration;

/// Registration flow descriptor for Kiro via AWS Builder ID.
#[apply(plain_default_copy_eq)]
pub struct KiroRegistrationFlow;

impl KiroRegistrationFlow {
    /// Kiro sign-in URL.
    pub const START_URL: &'static str = "https://app.kiro.dev/signin";

    /// Creates a Kiro flow descriptor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Polls the temp-mail inbox for a verification code.
    ///
    /// Uses the generic [`TempMailProvider`] trait so it works with any
    /// provider (mail.tm, cloudflare worker, etc.).
    fn poll_verification_code(
        provider: &dyn TempMailProvider,
        mailbox: &TempMailMailbox,
        max_wait: Duration,
        interval: Duration,
    ) -> Option<String> {
        let deadline = std::time::Instant::now() + max_wait;

        while std::time::Instant::now() < deadline {
            // List message summaries
            if let Ok(listing) = provider.list_messages(mailbox, PageRequest::new(20, 0)) {
                for summary in &listing.results {
                    // Fetch full message to get body text
                    if let Ok(Some(detail)) = provider.get_message(mailbox, &summary.id.to_string())
                    {
                        let combined = format!("{}\n{}", detail.subject, detail.text);
                        if let Some(code) = extract_verification_code(&combined) {
                            return Some(code);
                        }
                    }
                }
            }
            thread::sleep(interval);
        }
        None
    }

    /// Waits until the browser URL contains the given substring.
    fn wait_for_url(session: &BrowserSession, substring: &str, timeout: Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if let Ok(url) = session.execute_js("document.location.href")
                && let Some(s) = url.as_str()
                && s.contains(substring)
            {
                return true;
            }
            thread::sleep(Duration::from_millis(500));
        }
        false
    }

    /// Waits for an element matching a CSS selector to appear.
    fn wait_for_element(session: &BrowserSession, selector: &str, timeout: Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        let js = format!(
            "document.querySelector('{}') !== null",
            selector.replace('\'', "\\'")
        );
        while std::time::Instant::now() < deadline {
            if let Ok(val) = session.execute_js(&js)
                && val.as_bool() == Some(true)
            {
                return true;
            }
            thread::sleep(Duration::from_millis(300));
        }
        false
    }

    /// Types text character by character with random delays (human-like).
    fn human_type(
        session: &BrowserSession,
        selector: &str,
        text: &str,
    ) -> BrowserAutomationResult<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let focus_js = format!(
            "document.querySelector('{}').focus()",
            selector.replace('\'', "\\'")
        );
        session.execute_js(&focus_js).ok();

        for ch in text.chars() {
            let key_js = format!(
                r#"
                (() => {{
                    const el = document.querySelector('{sel}');
                    if (el) {{
                        el.value += {ch:?};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }}
                }})()
                "#,
                sel = selector.replace('\'', "\\'"),
            );
            session.execute_js(&key_js)?;
            thread::sleep(Duration::from_millis(rng.gen_range(50..180)));
        }
        Ok(())
    }

    /// Clicks an element by CSS selector.
    fn click_element(session: &BrowserSession, selector: &str) -> BrowserAutomationResult<()> {
        let js = format!(
            r#"
            (() => {{
                const el = document.querySelector('{sel}');
                if (el) {{ el.click(); return true; }}
                return false;
            }})()
            "#,
            sel = selector.replace('\'', "\\'")
        );
        let result = session.execute_js(&js)?;
        if result.as_bool() == Some(true) {
            Ok(())
        } else {
            Err(crate::browser_automation::BrowserAutomationError::Browser(
                format!("element not found: {selector}"),
            ))
        }
    }

    /// Clicks a button by visible text content.
    fn click_button_with_text(session: &BrowserSession, text: &str) -> BrowserAutomationResult<()> {
        let js = format!(
            r#"
            (() => {{
                const buttons = [...document.querySelectorAll('button, input[type="submit"], a[role="button"]')];
                const target = buttons.find(b => b.textContent.trim().includes('{text}'));
                if (target) {{ target.click(); return true; }}
                return false;
            }})()
            "#,
            text = text.replace('\'', "\\'")
        );
        let result = session.execute_js(&js)?;
        if result.as_bool() == Some(true) {
            Ok(())
        } else {
            Err(crate::browser_automation::BrowserAutomationError::Browser(
                format!("button with text '{text}' not found"),
            ))
        }
    }
}

impl RegistrationFlow for KiroRegistrationFlow {
    fn name(&self) -> &str {
        "kiro"
    }

    fn start_url(&self) -> &str {
        Self::START_URL
    }

    /// Executes the full Kiro registration flow.
    ///
    /// Steps:
    /// 1. Create a disposable mailbox via mail.tm
    /// 2. Navigate to Kiro sign-in page
    /// 3. Click "Builder ID Sign in"
    /// 4. Enter temp email, click Continue
    /// 5. Poll temp-mail for 6-digit verification code
    /// 6. Enter verification code, click Continue
    /// 7. Verify redirect to app.kiro.dev
    fn execute(
        &self,
        session: &BrowserSession,
        email: &str,
    ) -> BrowserAutomationResult<RegistrationResult> {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        // Step 1: Navigate to Kiro sign-in
        session.navigate(Self::START_URL)?;
        thread::sleep(Duration::from_millis(rng.gen_range(1000..2000)));

        let _ = session.screenshot("/tmp/kiro-step1-signin.png");

        // Step 2: Click Builder ID Sign in
        let builder_id_clicked = Self::click_button_with_text(session, "Builder ID").is_ok();
        if !builder_id_clicked {
            // Try via JS click on the specific button
            let js = r#"
            (() => {
                const btns = document.querySelectorAll('button');
                for (const b of btns) {
                    if (b.textContent.includes('Builder ID')) { b.click(); return true; }
                }
                return false;
            })()
            "#;
            let _ = session.execute_js(js)?;
        }

        thread::sleep(Duration::from_millis(rng.gen_range(2000..4000)));

        // Step 3: Wait for AWS Builder ID page
        let is_builder_id_page =
            Self::wait_for_url(
                session,
                "buildervoice.aws.amazon.com",
                Duration::from_secs(15),
            ) || Self::wait_for_url(session, "signin.aws.amazon.com", Duration::from_secs(5))
                || Self::wait_for_url(session, "id.aws.amazon.com", Duration::from_secs(5));

        if !is_builder_id_page
            && !Self::wait_for_element(session, "input[type='email']", Duration::from_secs(5))
        {
            let _ = session.screenshot("/tmp/kiro-step3-fail.png");
            return Ok(RegistrationResult::failure(
                "kiro",
                email,
                "Builder ID page did not load",
            ));
        }

        thread::sleep(Duration::from_millis(rng.gen_range(1000..2000)));
        let _ = session.screenshot("/tmp/kiro-step3-builderid.png");

        // Step 4: Enter email and click Continue
        let email_selectors = ["input[type='email']", "input[name='email']", "#email"];
        let email_filled = email_selectors.iter().any(|sel| {
            Self::wait_for_element(session, sel, Duration::from_secs(5))
                && Self::human_type(session, sel, email).is_ok()
        });

        if !email_filled {
            let _ = session.screenshot("/tmp/kiro-step4-fillfail.png");
            return Ok(RegistrationResult::failure(
                "kiro",
                email,
                "Failed to fill email field",
            ));
        }

        thread::sleep(Duration::from_millis(rng.gen_range(500..1500)));

        // Click Continue
        let _ = Self::click_button_with_text(session, "\u{7EE7}\u{7EED}") // 继续
            .or_else(|_| Self::click_button_with_text(session, "Continue"))
            .or_else(|_| Self::click_button_with_text(session, "Next"))
            .or_else(|_| Self::click_element(session, "button[type='submit']"));

        thread::sleep(Duration::from_millis(rng.gen_range(2000..4000)));
        let _ = session.screenshot("/tmp/kiro-step4-aftercontinue.png");

        // Step 5: Poll temp-mail for verification code
        let mail_provider = az_temp_mail::TempMail::mail_tm().map_err(|e| {
            crate::browser_automation::BrowserAutomationError::Browser(format!(
                "temp-mail init failed: {e}"
            ))
        })?;

        let code = Self::poll_verification_code(
            &mail_provider,
            // Use the passed-in email as a pseudo-mailbox identifier;
            // the real polling works through the provider's list_messages.
            &TempMailMailbox {
                provider: az_temp_mail::TempMailProviderKind::MailTm,
                address: email.to_owned(),
                credential: email.to_owned(),
                account_id: None,
                password: None,
            },
            Duration::from_secs(60),
            Duration::from_secs(3),
        );

        let code = match code {
            Some(c) => c,
            None => {
                let _ = session.screenshot("/tmp/kiro-step5-nocode.png");
                return Ok(RegistrationResult::failure(
                    "kiro",
                    email,
                    "Verification code not received within timeout",
                ));
            }
        };

        // Step 6: Enter verification code
        thread::sleep(Duration::from_millis(rng.gen_range(500..1500)));

        let code_selectors = [
            "input[type='text']",
            "input[type='number']",
            "input[name='code']",
            "input[name='otp_code']",
            "input[aria-label*='code']",
            "input[aria-label*='Code']",
            "input[aria-label*='verification']",
        ];
        let code_filled = code_selectors.iter().any(|sel| {
            Self::wait_for_element(session, sel, Duration::from_secs(5))
                && Self::human_type(session, sel, &code).is_ok()
        });

        if !code_filled {
            // Fallback: try any visible text/number input that is not email/password
            let fallback_js = format!(
                r#"
                (() => {{
                    const inputs = document.querySelectorAll('input[type="text"], input[type="number"], input:not([type])');
                    for (const el of inputs) {{
                        if (el.offsetParent !== null && !el.value) {{
                            el.focus();
                            el.value = '{code}';
                            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            return true;
                        }}
                    }}
                    return false;
                }})()
                "#,
                code = code
            );
            session.execute_js(&fallback_js).ok();
        }

        thread::sleep(Duration::from_millis(rng.gen_range(500..1500)));

        // Click Continue / Verify / Submit
        let _ = Self::click_button_with_text(session, "\u{7EE7}\u{7EED}") // 继续
            .or_else(|_| Self::click_button_with_text(session, "Continue"))
            .or_else(|_| Self::click_button_with_text(session, "Verify"))
            .or_else(|_| Self::click_button_with_text(session, "Submit"))
            .or_else(|_| Self::click_element(session, "button[type='submit']"));

        thread::sleep(Duration::from_millis(rng.gen_range(3000..5000)));
        let _ = session.screenshot("/tmp/kiro-step6-afterverify.png");

        // Step 7: Verify success — check if redirected to app.kiro.dev
        let mut success = Self::wait_for_url(session, "app.kiro.dev", Duration::from_secs(15));

        if !success {
            // May need to accept terms / agreements
            let _ = Self::click_button_with_text(session, "Accept")
                .or_else(|_| Self::click_button_with_text(session, "Agree"))
                .or_else(|_| Self::click_button_with_text(session, "I agree"));

            thread::sleep(Duration::from_millis(3000));
            success = Self::wait_for_url(session, "app.kiro.dev", Duration::from_secs(10));
        }

        let _ = session.screenshot("/tmp/kiro-step7-final.png");

        if success {
            Ok(RegistrationResult::success("kiro", email)
                .with_verification_code(code)
                .with_screenshot("/tmp/kiro-step7-final.png"))
        } else {
            Ok(RegistrationResult::failure(
                "kiro",
                email,
                "Registration did not complete — final redirect to app.kiro.dev not detected",
            )
            .with_verification_code(code)
            .with_screenshot("/tmp/kiro-step7-final.png"))
        }
    }
}
