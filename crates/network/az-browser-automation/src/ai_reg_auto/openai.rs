//! OpenAI login and sign-up page automation.

use crate::BrowserAutomationResult;
use crate::{BrowserAutomation, BrowserAutomationError, BrowserAutomationOptions};
use az_derive_aliases::{
    apply, deserialize_eq, plain_copy_eq, plain_copy_eq_hash, plain_default_copy_eq, plain_eq,
};
use az_temp_mail::{PageRequest, TempMailMailbox, TempMailProvider, create_mail_tm_api};
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::Runtime;
use serde_json::Value;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;

/// Sleeps a random duration between 5-70 seconds to mimic human pacing.
fn random_jitter() {
    let mut rng = rand::thread_rng();
    let secs = rng.gen_range(5..=70);
    thread::sleep(Duration::from_secs(secs));
}

/// OpenAI authorization flow mode.
#[apply(plain_copy_eq)]
pub enum OpenAiAuthFlow {
    /// Open the login flow.
    Login,
    /// Open the sign-up flow.
    SignUp,
}

/// Current stage reached by [`OpenAiAuthAutomation`].
#[apply(plain_copy_eq)]
pub enum OpenAiAuthStage {
    /// The authorization page was opened, but no credential input was requested.
    Opened,
    /// The email address was submitted.
    EmailSubmitted,
    /// A password field is present and no password was supplied.
    PasswordRequired,
    /// The password was submitted.
    PasswordSubmitted,
    /// The browser appears to have reached an authenticated OpenAI surface.
    Authenticated,
    /// OpenAI is asking for email, MFA, or similar human verification.
    VerificationRequired,
    /// A CAPTCHA or bot challenge is present.
    CaptchaRequired,
    /// The page reached a state the automation intentionally leaves to a human.
    AwaitingUserAction,
    /// Onboarding / "About You" page (name, age, etc.).
    OnboardingRequired,
    /// OpenAI rejected the registration based on Terms of Service.
    TermsRejected,
}

/// Step identifiers for manual OpenAI entry-flow recording.
#[apply(plain_copy_eq_hash)]
pub enum OpenAiRecordingStep {
    /// Step 1: open the OpenAI entry page.
    OpenEntryPage,
    /// Step 2: click the `Log in` entry point.
    ClickLogin,
    /// Step 3: click the `Sign up` / `Create account` entry from the login page.
    ClickSignUp,
}

impl OpenAiRecordingStep {
    /// Returns the stable step id used by manual recording helpers.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::OpenEntryPage => "step1",
            Self::ClickLogin => "step2",
            Self::ClickSignUp => "step3",
        }
    }

    /// Returns a short human-readable step title.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::OpenEntryPage => "Open entry page",
            Self::ClickLogin => "Click Log in",
            Self::ClickSignUp => "Click Sign up",
        }
    }

    /// Returns the intended manual recording action.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::OpenEntryPage => {
                "Open the ChatGPT/OpenAI entry page and wait for the shell to stabilize."
            }
            Self::ClickLogin => "Click the `Log in` entry button or link.",
            Self::ClickSignUp => {
                "On the login page, click `Sign up` or `Create account` for the `Don't have an account?` branch."
            }
        }
    }

    /// Parses `step1`, `step2`, or `step3`.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "step1" => Some(Self::OpenEntryPage),
            "step2" => Some(Self::ClickLogin),
            "step3" => Some(Self::ClickSignUp),
            _ => None,
        }
    }
}

/// Result status for one manual recording step.
#[apply(plain_copy_eq)]
pub enum OpenAiRecordingStepStatus {
    /// The target for the step was found and clicked, or the page was ready.
    Completed,
    /// The browser stayed open but the expected target for the step was not found.
    MissingTarget,
}

/// Result for one recorded step.
#[apply(plain_eq)]
pub struct OpenAiRecordingStepResult {
    /// Step that was executed.
    pub step: OpenAiRecordingStep,
    /// Step status.
    pub status: OpenAiRecordingStepStatus,
    /// Final observed browser URL.
    pub final_url: String,
    /// Final observed page title.
    pub page_title: String,
    /// Human-readable step outcome.
    pub message: String,
}

/// Result for a multi-step manual recording run.
#[apply(plain_eq)]
pub struct OpenAiRecordingResult {
    /// Executed step results in order.
    pub steps: Vec<OpenAiRecordingStepResult>,
}

impl OpenAiRecordingResult {
    /// Returns true when every requested step completed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.steps
            .iter()
            .all(|step| step.status == OpenAiRecordingStepStatus::Completed)
    }
}

/// Options for manual OpenAI entry-flow recording.
#[apply(plain_eq)]
pub struct OpenAiRecordingOptions {
    /// Initial page used for the recording sequence.
    pub start_url: String,
    /// Ordered steps to execute.
    pub steps: Vec<OpenAiRecordingStep>,
    /// Delay between steps.
    pub step_delay: Duration,
    /// Optional pause after each step.
    pub hold_after_each_step: Option<Duration>,
    /// Optional pause after the final step.
    pub hold_after_finish: Option<Duration>,
}

impl Default for OpenAiRecordingOptions {
    fn default() -> Self {
        Self::entry_sign_up()
    }
}

impl OpenAiRecordingOptions {
    /// Default landing page used for entry-flow recording.
    pub const ENTRY_URL: &'static str = "https://chatgpt.com/";

    /// Returns the default `step1 -> step2 -> step3` recording plan.
    #[must_use]
    pub fn entry_sign_up() -> Self {
        Self {
            start_url: Self::ENTRY_URL.to_owned(),
            steps: vec![
                OpenAiRecordingStep::OpenEntryPage,
                OpenAiRecordingStep::ClickLogin,
                OpenAiRecordingStep::ClickSignUp,
            ],
            step_delay: Duration::from_millis(900),
            hold_after_each_step: None,
            hold_after_finish: None,
        }
    }

    /// Replaces the step sequence.
    #[must_use]
    pub fn with_steps(mut self, steps: impl IntoIterator<Item = OpenAiRecordingStep>) -> Self {
        self.steps = steps.into_iter().collect();
        self
    }

    /// Sets an explicit start URL.
    #[must_use]
    pub fn with_start_url(mut self, start_url: impl Into<String>) -> Self {
        self.start_url = normalize_page_url(start_url.into());
        self
    }

    /// Sets the delay between steps.
    #[must_use]
    pub fn with_step_delay(mut self, step_delay: Duration) -> Self {
        self.step_delay = step_delay;
        self
    }

    /// Pauses after every step.
    #[must_use]
    pub fn with_hold_after_each_step(mut self, hold_after_each_step: Duration) -> Self {
        self.hold_after_each_step = Some(hold_after_each_step);
        self
    }

    /// Keeps the final page open after the recording plan finishes.
    #[must_use]
    pub fn with_hold_after_finish(mut self, hold_after_finish: Duration) -> Self {
        self.hold_after_finish = Some(hold_after_finish);
        self
    }
}

/// Options for automating the OpenAI authorization entry flow.
#[apply(plain_eq)]
pub struct OpenAiAuthOptions {
    /// Initial URL for the OpenAI authorization flow.
    pub start_url: String,
    /// Login or sign-up mode.
    pub flow: OpenAiAuthFlow,
    /// Optional email address to type into the authorization form.
    pub email: Option<String>,
    /// Optional password to submit if the provider asks for one.
    pub password: Option<String>,
    /// Delay between page actions, useful when recording a headful session.
    pub step_delay: Duration,
    /// Optional time to keep the tab open after the automation reaches a stop stage.
    pub hold_for: Option<Duration>,
}

impl Default for OpenAiAuthOptions {
    fn default() -> Self {
        Self::open_login()
    }
}

impl OpenAiAuthOptions {
    /// Default OpenAI login URL.
    pub const LOGIN_URL: &'static str = "https://auth.openai.com/log-in";
    /// Default OpenAI sign-up URL.
    pub const SIGN_UP_URL: &'static str = "https://auth.openai.com/create-account";

    /// Creates options that only open the login page.
    #[must_use]
    pub fn open_login() -> Self {
        Self {
            start_url: Self::LOGIN_URL.to_owned(),
            flow: OpenAiAuthFlow::Login,
            email: None,
            password: None,
            step_delay: Duration::from_millis(700),
            hold_for: None,
        }
    }

    /// Creates a login flow with an email address.
    #[must_use]
    pub fn login(email: impl Into<String>) -> Self {
        Self {
            email: Some(email.into()),
            ..Self::open_login()
        }
    }

    /// Creates a sign-up flow with an email address.
    #[must_use]
    pub fn sign_up(email: impl Into<String>) -> Self {
        Self {
            start_url: Self::SIGN_UP_URL.to_owned(),
            flow: OpenAiAuthFlow::SignUp,
            email: Some(email.into()),
            ..Self::open_login()
        }
    }

    /// Sets an explicit start URL.
    #[must_use]
    pub fn with_start_url(mut self, start_url: impl Into<String>) -> Self {
        self.start_url = normalize_page_url(start_url.into());
        self
    }

    /// Sets the password used if the flow asks for one.
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the delay between automation steps.
    #[must_use]
    pub fn with_step_delay(mut self, delay: Duration) -> Self {
        self.step_delay = delay;
        self
    }

    /// Keeps the tab open after the automation stops.
    #[must_use]
    pub fn with_hold_for(mut self, hold_for: Duration) -> Self {
        self.hold_for = Some(hold_for);
        self
    }
}

/// Result returned by [`OpenAiAuthAutomation`].
#[apply(plain_eq)]
pub struct OpenAiAuthResult {
    /// Stage where automation stopped.
    pub stage: OpenAiAuthStage,
    /// Final observed browser URL.
    pub final_url: String,
    /// Final observed page title.
    pub page_title: String,
    /// Human-readable stop reason.
    pub message: String,
}

impl OpenAiAuthResult {
    /// Returns true when the browser appears authenticated.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.stage == OpenAiAuthStage::Authenticated
    }
}

/// OpenAI authorization page automation.
#[apply(plain_default_copy_eq)]
pub struct OpenAiAuthAutomation;

/// If the page shows a "session ended" notice, click "Log in" to
/// dismiss it and reach the real auth form.
fn dismiss_session_ended(tab: &Arc<Tab>) -> BrowserAutomationResult<bool> {
    let body = evaluate_json(
        tab,
        "document.body ? document.body.innerText.slice(0, 500) : ''",
    )?;
    let text = body.as_str().unwrap_or("").to_lowercase();
    if text.contains("session ended")
        || text.contains("会话已结束")
        || text.contains("your session")
    {
        click_button_by_text(tab, &["Log in", "Login", "登录", "log in"])
    } else {
        Ok(false)
    }
}

impl OpenAiAuthAutomation {
    /// Runs a manual entry-flow recording plan.
    ///
    /// The default plan is:
    /// `step1` open entry page -> `step2` click `Log in` -> `step3` click
    /// `Sign up`.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when browser connection, navigation,
    /// or JavaScript evaluation fails.
    pub fn record(
        recording_options: &OpenAiRecordingOptions,
        browser_options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<OpenAiRecordingResult> {
        BrowserAutomation::with_tab(&recording_options.start_url, browser_options, |tab| {
            let mut steps = Vec::with_capacity(recording_options.steps.len());

            for step in &recording_options.steps {
                thread::sleep(recording_options.step_delay);
                let step_result = run_recording_step(tab, *step)?;
                let completed = step_result.status == OpenAiRecordingStepStatus::Completed;
                steps.push(step_result);

                if let Some(hold_after_each_step) = recording_options.hold_after_each_step {
                    thread::sleep(hold_after_each_step);
                }

                if !completed {
                    break;
                }
            }

            if let Some(hold_after_finish) = recording_options.hold_after_finish {
                thread::sleep(hold_after_finish);
            }

            Ok(OpenAiRecordingResult { steps })
        })
    }

    /// Runs the configured OpenAI authorization flow in a browser tab.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when browser connection, navigation,
    /// JavaScript evaluation, or page interaction fails.
    pub fn run(
        auth_options: &OpenAiAuthOptions,
        browser_options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<OpenAiAuthResult> {
        BrowserAutomation::with_tab(&auth_options.start_url, browser_options, |tab| {
            let result = Self::run_on_tab(tab, auth_options)?;
            if let Some(hold_for) = auth_options.hold_for {
                thread::sleep(hold_for);
            }
            Ok(result)
        })
    }

    /// If the page shows a "session ended" notice, click "Log in" to
    /// dismiss it and reach the real auth form.
    fn dismiss_session_ended(tab: &Arc<Tab>) -> BrowserAutomationResult<bool> {
        let body = evaluate_json(
            tab,
            "document.body ? document.body.innerText.slice(0, 500) : ''",
        )?;
        let text = body.as_str().unwrap_or("").to_lowercase();
        if text.contains("session ended")
            || text.contains("会话已结束")
            || text.contains("your session")
        {
            // Click "Log in" / "登录" to reset the stale session
            click_button_by_text(tab, &["Log in", "Login", "登录", "log in"])
        } else {
            Ok(false)
        }
    }

    fn run_on_tab(
        tab: &Arc<Tab>,
        auth_options: &OpenAiAuthOptions,
    ) -> BrowserAutomationResult<OpenAiAuthResult> {
        thread::sleep(auth_options.step_delay);

        // Clear any stale session first (OpenAI may show "session ended")
        let _ = dismiss_session_ended(tab);
        thread::sleep(auth_options.step_delay);

        if auth_options.flow == OpenAiAuthFlow::SignUp {
            eprintln!("[DEBUG] attempting to click submit button...");
            let _ = click_button_by_text(
                tab,
                &[
                    "Sign up",
                    "Create account",
                    "注册",
                    "Create",
                    "Get started",
                    "Don\'t have",
                    "sign up",
                    "Create",
                    "Get started",
                    "Start",
                    "Register",
                    "Don\'t have",
                ],
            );
            thread::sleep(auth_options.step_delay);
        }

        let mut state = read_state(tab)?;
        if state.is_authenticated() {
            return Ok(result(
                OpenAiAuthStage::Authenticated,
                state,
                "OpenAI authenticated surface detected",
            ));
        }

        let Some(email) = auth_options.email.as_deref() else {
            return Ok(result(
                OpenAiAuthStage::Opened,
                state,
                "OpenAI auth page opened; no email was supplied",
            ));
        };

        if !fill_first_visible(tab, EMAIL_SELECTORS, email)? {
            return Ok(result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "email input was not found",
            ));
        }

        if !click_continue(tab)? {
            return Ok(result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "email was filled, but the submit button was not found",
            ));
        }
        state = wait_for_state(tab, Duration::from_secs(20), |state| {
            state.is_authenticated()
                || state.has_password_input
                || state.has_verification
                || state.has_captcha
        })?;

        if state.is_authenticated() {
            return Ok(result(
                OpenAiAuthStage::Authenticated,
                state,
                "OpenAI authenticated surface detected after email submit",
            ));
        }
        if state.has_captcha {
            return Ok(result(
                OpenAiAuthStage::CaptchaRequired,
                state,
                "OpenAI displayed a CAPTCHA or bot challenge",
            ));
        }
        if state.has_verification && !state.has_password_input {
            return Ok(result(
                OpenAiAuthStage::VerificationRequired,
                state,
                "OpenAI displayed a verification step",
            ));
        }

        if !state.has_password_input {
            return Ok(result(
                OpenAiAuthStage::EmailSubmitted,
                state,
                "email submitted; waiting for the next provider step",
            ));
        }

        let Some(password) = auth_options.password.as_deref() else {
            return Ok(result(
                OpenAiAuthStage::PasswordRequired,
                state,
                "password field is visible, but no password was supplied",
            ));
        };

        if !fill_first_visible(tab, PASSWORD_SELECTORS, password)? {
            return Ok(result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "password input was visible but could not be filled",
            ));
        }

        if !click_continue(tab)? {
            return Ok(result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "password was filled, but the submit button was not found",
            ));
        }
        state = wait_for_state(tab, Duration::from_secs(25), |state| {
            state.is_authenticated() || state.has_verification || state.has_captcha
        })?;

        if state.is_authenticated() {
            Ok(result(
                OpenAiAuthStage::Authenticated,
                state,
                "OpenAI authenticated surface detected after password submit",
            ))
        } else if state.has_captcha {
            Ok(result(
                OpenAiAuthStage::CaptchaRequired,
                state,
                "OpenAI displayed a CAPTCHA or bot challenge",
            ))
        } else if state.has_verification {
            Ok(result(
                OpenAiAuthStage::VerificationRequired,
                state,
                "OpenAI displayed a verification step",
            ))
        } else {
            Ok(result(
                OpenAiAuthStage::PasswordSubmitted,
                state,
                "password submitted; no completion signal detected yet",
            ))
        }
    }
}

const EMAIL_SELECTORS: &[&str] = &[
    "input[type='email']",
    "input[name='email']",
    "input[name='username']",
    "input[autocomplete='email']",
    "input[autocomplete='username']",
    "input[id*='email' i]",
    "input[id*='username' i]",
    "input[placeholder*='email' i]",
    "input[placeholder*='address' i]",
    "input[aria-label*='email' i]",
    "input[data-testid*='email' i]",
    "input:not([type='hidden']):not([type='submit']):not([type='button'])",
    "input[type='text']:not([readonly]):not([disabled])",
];

const PASSWORD_SELECTORS: &[&str] = &[
    "input[type='password']",
    "input[name='password']",
    "input[autocomplete='current-password']",
    "input[autocomplete='new-password']",
    "input[id*='password' i]",
    "input[placeholder*='password' i]",
    "input[aria-label*='password' i]",
    "input[data-testid*='password' i]",
];

#[apply(deserialize_eq)]
#[serde(rename_all = "camelCase")]
struct AuthPageState {
    url: String,
    title: String,
    has_password_input: bool,
    has_verification: bool,
    has_captcha: bool,
    #[serde(default)]
    has_onboarding: bool,
    #[serde(default)]
    has_terms_rejected: bool,
}

impl AuthPageState {
    fn is_authenticated(&self) -> bool {
        let url = self.url.to_ascii_lowercase();
        url.contains("platform.openai.com")
            || url.contains("chatgpt.com")
            || url.contains("chat.openai.com")
        // `about-you` is onboarding, NOT authenticated — handled separately
    }
}

fn run_recording_step(
    tab: &Arc<Tab>,
    step: OpenAiRecordingStep,
) -> BrowserAutomationResult<OpenAiRecordingStepResult> {
    match step {
        OpenAiRecordingStep::OpenEntryPage => {
            let state = read_state(tab)?;
            Ok(recording_result(
                step,
                OpenAiRecordingStepStatus::Completed,
                state,
                "entry page is open",
            ))
        }
        OpenAiRecordingStep::ClickLogin => {
            let clicked = click_button_by_text(tab, &["Log in", "Login"])?;
            thread::sleep(Duration::from_millis(1_000));
            let state = read_state(tab)?;
            Ok(if clicked {
                recording_result(
                    step,
                    OpenAiRecordingStepStatus::Completed,
                    state,
                    "clicked the `Log in` entry point",
                )
            } else {
                recording_result(
                    step,
                    OpenAiRecordingStepStatus::MissingTarget,
                    state,
                    "the `Log in` entry point was not found",
                )
            })
        }
        OpenAiRecordingStep::ClickSignUp => {
            let clicked = click_button_by_text(tab, &["Sign up", "Create account", "Register"])?;
            thread::sleep(Duration::from_millis(1_000));
            let state = read_state(tab)?;
            Ok(if clicked {
                recording_result(
                    step,
                    OpenAiRecordingStepStatus::Completed,
                    state,
                    "clicked the `Sign up` / `Create account` branch",
                )
            } else {
                recording_result(
                    step,
                    OpenAiRecordingStepStatus::MissingTarget,
                    state,
                    "the `Sign up` / `Create account` branch was not found",
                )
            })
        }
    }
}

fn result(stage: OpenAiAuthStage, state: AuthPageState, message: &str) -> OpenAiAuthResult {
    OpenAiAuthResult {
        stage,
        final_url: state.url,
        page_title: state.title,
        message: message.to_owned(),
    }
}

fn recording_result(
    step: OpenAiRecordingStep,
    status: OpenAiRecordingStepStatus,
    state: AuthPageState,
    message: &str,
) -> OpenAiRecordingStepResult {
    OpenAiRecordingStepResult {
        step,
        status,
        final_url: state.url,
        page_title: state.title,
        message: message.to_owned(),
    }
}

fn click_continue(tab: &Arc<Tab>) -> BrowserAutomationResult<bool> {
    if click_button_by_text(
        tab,
        &[
            "Continue",
            "Log in",
            "Login",
            "Next",
            "Sign up",
            "Create account",
            "Verify",
        ],
    )? || click_first_visible(tab, &["button[type='submit']", "input[type='submit']"])?
    {
        return Ok(true);
    }

    Ok(false)
}

fn wait_for_state(
    tab: &Arc<Tab>,
    timeout: Duration,
    accept: impl Fn(&AuthPageState) -> bool,
) -> BrowserAutomationResult<AuthPageState> {
    let deadline = Instant::now() + timeout;
    let mut last_state = read_state(tab)?;

    while Instant::now() < deadline {
        if accept(&last_state) {
            return Ok(last_state);
        }
        thread::sleep(Duration::from_millis(500));
        last_state = read_state(tab)?;
    }

    Ok(last_state)
}

fn read_state(tab: &Arc<Tab>) -> BrowserAutomationResult<AuthPageState> {
    let value = evaluate_json(
        tab,
        r#"
        (() => {
            const visible = (el) => {
                const style = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                return style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0
                    && rect.height > 0;
            };
            const inputs = [...document.querySelectorAll('input, textarea')].filter(visible);
            const bodyText = document.body ? document.body.innerText.toLowerCase() : '';
            const hasPasswordInput = inputs.some((el) => {
                const haystack = [
                    el.type,
                    el.name,
                    el.id,
                    el.autocomplete,
                    el.placeholder,
                    el.getAttribute('aria-label')
                ].join(' ').toLowerCase();
                return haystack.includes('password');
            });
            const href = window.location.href.toLowerCase();
            const hasOnboarding = href.includes('about-you')
                || href.includes('onboarding')
                || bodyText.includes('about you')
                || bodyText.includes('完成帐户')
                || bodyText.includes('完成账户');

            const hasVerification = href.includes('verify')
                || href.includes('verification')
                || href.includes('code')
                || [
                    'verify',
                    'verification',
                    'check your email',
                    'security code',
                    'multi-factor',
                    'two-factor',
                    'authenticator',
                    'one-time code',
                    '验证',
                    '验证码',
                    '邮箱验证',
                    '安全码',
                ].some((token) => bodyText.includes(token));
            const hasTermsRejected = bodyText.includes('使用条款')
                || bodyText.includes('terms of use')
                || bodyText.includes('terms of service')
                || bodyText.includes('cannot create')
                || bodyText.includes('无法创建');
            const hasCaptcha = bodyText.includes('captcha')
                || document.querySelector('[class*="captcha" i], [id*="captcha" i], iframe[src*="captcha" i], iframe[src*="hcaptcha" i], iframe[src*="recaptcha" i], iframe[src*="turnstile" i]') !== null;
            return {
                url: window.location.href,
                title: document.title,
                hasPasswordInput,
                hasVerification,
                hasCaptcha,
                hasOnboarding,
                hasTermsRejected
            };
        })()
        "#,
    )?;
    serde_json::from_value(value).map_err(to_browser_error)
}

fn fill_first_visible(
    tab: &Arc<Tab>,
    selectors: &[&str],
    value: &str,
) -> BrowserAutomationResult<bool> {
    let selectors = js_value(selectors)?;
    let value = js_value(value)?;
    let script = format!(
        r#"
        (() => {{
            const selectors = {selectors};
            const value = {value};
            const visible = (el) => {{
                const style = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                return !el.disabled
                    && !el.readOnly
                    && style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0
                    && rect.height > 0;
            }};
            for (const selector of selectors) {{
                for (const el of document.querySelectorAll(selector)) {{
                    if (!visible(el) || !('value' in el)) {{
                        continue;
                    }}
                    el.focus();
                    const descriptor = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value');
                    if (descriptor && descriptor.set) {{
                        descriptor.set.call(el, value);
                    }} else {{
                        el.value = value;
                    }}
                    el.dispatchEvent(new InputEvent('input', {{ bubbles: true, data: value, inputType: 'insertText' }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }}
            }}
            return false;
        }})()
        "#
    );
    let click_result = evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false));
    eprintln!("[DEBUG] click_button_by_text result={:?}", click_result);
    click_result
}

fn click_button_by_text(tab: &Arc<Tab>, labels: &[&str]) -> BrowserAutomationResult<bool> {
    let labels = js_value(labels)?;
    let script = format!(
        r#"
        (() => {{
            const labels = {labels}.map((label) => label.toLowerCase());
            const visible = (el) => {{
                const style = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                return !el.disabled
                    && style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0
                    && rect.height > 0;
            }};
            const nodes = [...document.querySelectorAll('button, a, [role="button"], input[type="button"], input[type="submit"]')];
            const target = nodes.find((el) => {{
                if (!visible(el)) {{
                    return false;
                }}
                const text = [
                    el.innerText,
                    el.textContent,
                    el.value,
                    el.getAttribute('aria-label')
                ].join(' ').trim().toLowerCase();
                return labels.some((label) => text === label || text.includes(label));
            }});
            if (!target) {{
                return false;
            }}
            target.click();
            return true;
        }})()
        "#
    );
    let click_result = evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false));
    eprintln!("[DEBUG] click_button_by_text result={:?}", click_result);
    click_result
}

fn click_first_visible(tab: &Arc<Tab>, selectors: &[&str]) -> BrowserAutomationResult<bool> {
    let selectors = js_value(selectors)?;
    let script = format!(
        r#"
        (() => {{
            const selectors = {selectors};
            const visible = (el) => {{
                const style = window.getComputedStyle(el);
                const rect = el.getBoundingClientRect();
                return !el.disabled
                    && style.visibility !== 'hidden'
                    && style.display !== 'none'
                    && rect.width > 0
                    && rect.height > 0;
            }};
            for (const selector of selectors) {{
                const target = [...document.querySelectorAll(selector)].find(visible);
                if (target) {{
                    target.click();
                    return true;
                }}
            }}
            return false;
        }})()
        "#
    );
    let click_result = evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false));
    eprintln!("[DEBUG] click_button_by_text result={:?}", click_result);
    click_result
}

fn evaluate_json(tab: &Arc<Tab>, js: &str) -> BrowserAutomationResult<Value> {
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
        .map_err(to_browser_error)?;
    if let Some(exception) = result.exception_details {
        return Err(BrowserAutomationError::Browser(format!(
            "JavaScript evaluation failed: {exception:?}"
        )));
    }
    Ok(result.result.value.unwrap_or(Value::Null))
}

fn js_value<T: serde::Serialize>(value: T) -> BrowserAutomationResult<String> {
    serde_json::to_string(&value).map_err(to_browser_error)
}

fn to_browser_error(error: impl ToString) -> BrowserAutomationError {
    BrowserAutomationError::Browser(error.to_string())
}

fn normalize_page_url(start_url: String) -> String {
    let trimmed = start_url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else {
        format!("https://{trimmed}")
    }
}

// ═══════════════════════════════════════════════════════════════════
// Full Registration Flow — temp_mail + SMS integration
// ═══════════════════════════════════════════════════════════════════

/// Configuration for a complete OpenAI signup that handles email verification
/// via a disposable mailbox and phone verification via 5sim SMS.
#[apply(plain_eq)]
pub struct OpenAiFullRegOptions {
    /// Start URL (defaults to OpenAI sign-up page).
    pub start_url: String,
    /// Password to use during sign-up. Auto-generated if not provided.
    pub password: Option<String>,
    /// Delay between automation steps.
    pub step_delay: Duration,
    /// Optional hold after the flow reaches a terminal stage.
    pub hold_for: Option<Duration>,
    /// 5sim API token for purchasing SMS verification numbers.
    pub sms_token: Option<String>,
    /// 5sim product name (e.g. `"openai"`).
    pub sms_product: String,
    /// 5sim country code (e.g. `"usa"`).
    pub sms_country: String,
    /// 5sim operator code (e.g. `"any"`).
    pub sms_operator: String,
    /// Prefix for the disposable email local part.
    pub email_prefix: String,
}

impl Default for OpenAiFullRegOptions {
    fn default() -> Self {
        Self {
            start_url: OpenAiAuthOptions::SIGN_UP_URL.to_owned(),
            password: None,
            step_delay: Duration::from_millis(700),
            hold_for: None,
            sms_token: None,
            sms_product: "openai".to_owned(),
            sms_country: "usa".to_owned(),
            sms_operator: "any".to_owned(),
            email_prefix: "azit".to_owned(),
        }
    }
}

/// Result of a complete OpenAI signup attempt.
#[apply(plain_eq)]
pub struct OpenAiFullRegResult {
    /// Created disposable email address.
    pub email: String,
    /// Disposable email password (mail.tm credential).
    pub email_password: String,
    /// JWT token for the disposable mailbox.
    pub jwt_token: String,
    /// Password used for the OpenAI account.
    pub openai_password: String,
    /// Final automation stage.
    pub stage: OpenAiAuthStage,
    /// Final browser URL after the flow.
    pub final_url: String,
    /// Final page title.
    pub page_title: String,
    /// Human-readable outcome.
    pub message: String,
    /// Phone number purchased for SMS verification (if any).
    pub sms_phone: Option<String>,
    /// 5sim SMS order ID (if phone verification was used).
    pub sms_order_id: Option<u64>,
}

/// Complete OpenAI registration automation.
///
/// This wraps [`OpenAiAuthAutomation`] and adds:
/// - disposable email creation via [`az_temp_mail`]
/// - email verification code polling via the temp mailbox
/// - phone verification via [`az_sms`] (5sim)
#[apply(plain_default_copy_eq)]
pub struct OpenAiRegAutomation;

impl OpenAiRegAutomation {
    /// Runs the full OpenAI sign-up loop with disposable email and optional
    /// SMS verification.
    ///
    /// # Flow
    ///
    /// 1. Create a disposable mailbox on mail.tm
    /// 2. Open the sign-up page, fill email, continue
    /// 3. If email verification is required →
    ///    poll the disposable mailbox for the code, enter it
    /// 4. Fill password (auto-generated if none provided)
    /// 5. If phone verification is required →
    ///    buy a 5sim number, enter it, poll for SMS code, enter it
    /// 6. Return the final result
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when browser automation, temp-mail
    /// creation, SMS purchasing, or page interaction fails.
    pub fn run_full_registration(
        reg_options: &OpenAiFullRegOptions,
        browser_options: &BrowserAutomationOptions,
    ) -> BrowserAutomationResult<OpenAiFullRegResult> {
        // 1. Create disposable mailbox
        let api = create_mail_tm_api().map_err(to_browser_error)?;
        let mailbox = api
            .create_mailbox_and_login(&reg_options.email_prefix, 16)
            .map_err(to_browser_error)?;
        let openai_password = reg_options
            .password
            .clone()
            .unwrap_or_else(|| random_ascii_string(16));

        let email = mailbox.address.clone();
        let email_password = mailbox.password.clone().unwrap_or_default();
        let jwt = mailbox.credential.clone();

        let auth_options = OpenAiAuthOptions {
            start_url: reg_options.start_url.clone(),
            flow: OpenAiAuthFlow::SignUp,
            email: Some(email.clone()),
            password: Some(openai_password.clone()),
            step_delay: reg_options.step_delay,
            hold_for: None,
        };

        // 2–5. Run browser auth with email verification polling
        let (mut auth_result, sms_phone, sms_order_id) = Self::run_with_verification(
            &auth_options,
            browser_options,
            &api,
            &mailbox,
            reg_options,
        )?;

        // 6. Build final result
        let result = OpenAiFullRegResult {
            email,
            email_password,
            jwt_token: jwt,
            openai_password,
            stage: auth_result.stage,
            final_url: std::mem::take(&mut auth_result.final_url),
            page_title: std::mem::take(&mut auth_result.page_title),
            message: std::mem::take(&mut auth_result.message),
            sms_phone,
            sms_order_id,
        };

        if let Some(hold_for) = reg_options.hold_for {
            thread::sleep(hold_for);
        }

        Ok(result)
    }

    /// Runs the full registration flow on an existing [`BrowserSession`].
    ///
    /// This bypasses [`BrowserAutomation::with_tab`] and uses the caller-owned
    /// session tab directly, allowing fingerprint profiles, proxies, and
    /// isolated Chrome processes to be managed externally.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when browser automation, temp-mail
    /// creation, SMS purchasing, or page interaction fails.
    pub fn run_on_session(
        reg_options: &OpenAiFullRegOptions,
        session: &crate::BrowserSession,
    ) -> BrowserAutomationResult<OpenAiFullRegResult> {
        let api = create_mail_tm_api().map_err(to_browser_error)?;
        let mailbox = api
            .create_mailbox_and_login(&reg_options.email_prefix, 16)
            .map_err(to_browser_error)?;
        let openai_password = reg_options
            .password
            .clone()
            .unwrap_or_else(|| random_ascii_string(16));

        let email = mailbox.address.clone();
        let email_password = mailbox.password.clone().unwrap_or_default();
        let jwt = mailbox.credential.clone();

        let auth_options = OpenAiAuthOptions {
            start_url: reg_options.start_url.clone(),
            flow: OpenAiAuthFlow::SignUp,
            email: Some(email.clone()),
            password: Some(openai_password.clone()),
            step_delay: reg_options.step_delay,
            hold_for: None,
        };

        // Navigate to start URL before running verification on existing tab
        session
            .navigate(&auth_options.start_url)
            .map_err(to_browser_error)?;

        let (mut auth_result, sms_phone, sms_order_id) = Self::run_verification_on_tab(
            session.tab(),
            &auth_options,
            &api,
            &mailbox,
            reg_options,
        )?;

        let result = OpenAiFullRegResult {
            email,
            email_password,
            jwt_token: jwt,
            openai_password,
            stage: auth_result.stage,
            final_url: std::mem::take(&mut auth_result.final_url),
            page_title: std::mem::take(&mut auth_result.page_title),
            message: std::mem::take(&mut auth_result.message),
            sms_phone,
            sms_order_id,
        };

        if let Some(hold_for) = reg_options.hold_for {
            thread::sleep(hold_for);
        }

        Ok(result)
    }

    fn run_with_verification(
        auth_options: &OpenAiAuthOptions,
        browser_options: &BrowserAutomationOptions,
        provider: &dyn TempMailProvider,
        mailbox: &TempMailMailbox,
        reg_options: &OpenAiFullRegOptions,
    ) -> BrowserAutomationResult<(OpenAiAuthResult, Option<String>, Option<u64>)> {
        BrowserAutomation::with_tab(&auth_options.start_url, browser_options, |tab| {
            Self::run_verification_on_tab(tab, auth_options, provider, mailbox, reg_options)
        })
    }

    /// Core verification logic extracted so it can be called from both
    /// [`BrowserAutomation::with_tab`] and [`BrowserSession`] paths.
    fn run_verification_on_tab(
        tab: &Arc<Tab>,
        auth_options: &OpenAiAuthOptions,
        provider: &dyn TempMailProvider,
        mailbox: &TempMailMailbox,
        reg_options: &OpenAiFullRegOptions,
    ) -> BrowserAutomationResult<(OpenAiAuthResult, Option<String>, Option<u64>)> {
        thread::sleep(auth_options.step_delay);

        // Clear stale session first
        let _ = dismiss_session_ended(tab);
        thread::sleep(auth_options.step_delay);
        random_jitter();

        // Click "Sign up" / "Create account" if on the login page.
        eprintln!("[DEBUG] attempting to click submit button...");
        let _ = click_button_by_text(
            tab,
            &[
                "Sign up",
                "Create account",
                "注册",
                "Create",
                "Get started",
                "Don\\'t have",
            ],
        );
        thread::sleep(auth_options.step_delay);
        random_jitter();

        let mut state = read_state(tab)?;
        if state.is_authenticated() {
            let r = result(
                OpenAiAuthStage::Authenticated,
                state,
                "OpenAI already authenticated",
            );
            return Ok((r, None, None));
        }

        // ── fill email ──
        let email = auth_options.email.as_deref().unwrap_or("");
        if !fill_first_visible(tab, EMAIL_SELECTORS, email)? {
            let r = result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "email input not found",
            );
            return Ok((r, None, None));
        }
        if !click_continue(tab)? {
            let r = result(
                OpenAiAuthStage::AwaitingUserAction,
                state,
                "submit after email not found",
            );
            return Ok((r, None, None));
        }
        random_jitter();

        state = wait_for_state(tab, Duration::from_secs(25), |s| {
            s.is_authenticated() || s.has_password_input || s.has_verification || s.has_captcha
        })?;

        if state.is_authenticated() {
            let r = result(
                OpenAiAuthStage::Authenticated,
                state,
                "authenticated after email",
            );
            return Ok((r, None, None));
        }
        if state.has_captcha {
            let r = result(
                OpenAiAuthStage::CaptchaRequired,
                state,
                "captcha after email",
            );
            return Ok((r, None, None));
        }

        // ── email verification code ──
        if state.has_verification && !state.has_password_input {
            state =
                Self::handle_email_verification(tab, provider, mailbox, auth_options.step_delay)?;
            if state.is_authenticated() {
                let r = result(
                    OpenAiAuthStage::Authenticated,
                    state,
                    "authenticated after email verification",
                );
                return Ok((r, None, None));
            }

            if state.has_captcha {
                let r = result(
                    OpenAiAuthStage::CaptchaRequired,
                    state,
                    "captcha after email verification",
                );
                return Ok((r, None, None));
            }
        }

        // ── fill password ──
        if state.has_password_input {
            let password = auth_options.password.as_deref().unwrap_or("");
            if !fill_first_visible(tab, PASSWORD_SELECTORS, password)? {
                let r = result(
                    OpenAiAuthStage::AwaitingUserAction,
                    state,
                    "password input visible but could not be filled",
                );
                return Ok((r, None, None));
            }
            if !click_continue(tab)? {
                let r = result(
                    OpenAiAuthStage::AwaitingUserAction,
                    state,
                    "submit after password not found",
                );
                return Ok((r, None, None));
            }
            random_jitter();

            state = wait_for_state(tab, Duration::from_secs(25), |s| {
                s.is_authenticated() || s.has_verification || s.has_captcha || s.has_onboarding
            })?;

            if state.is_authenticated() {
                let r = result(
                    OpenAiAuthStage::Authenticated,
                    state,
                    "authenticated after password",
                );
                return Ok((r, None, None));
            }
            if state.has_captcha {
                let r = result(
                    OpenAiAuthStage::CaptchaRequired,
                    state,
                    "captcha after password",
                );
                return Ok((r, None, None));
            }
        }

        // ── terms rejected ──
        if state.has_terms_rejected {
            let r = result(
                OpenAiAuthStage::TermsRejected,
                state,
                "OpenAI rejected registration based on Terms of Service",
            );
            return Ok((r, None, None));
        }

        // ── onboarding (About You: name + age) ──
        eprintln!(
            "[DEBUG] has_onboarding={} has_password_input={} has_verification={} url={}",
            state.has_onboarding, state.has_password_input, state.has_verification, state.url
        );
        if state.has_onboarding {
            random_jitter();
            state = Self::handle_onboarding(tab, auth_options.step_delay)?;
            if state.is_authenticated() {
                let r = result(
                    OpenAiAuthStage::Authenticated,
                    state,
                    "authenticated after onboarding",
                );
                return Ok((r, None, None));
            }
            if state.has_terms_rejected {
                let r = result(
                    OpenAiAuthStage::TermsRejected,
                    state,
                    "OpenAI rejected registration based on Terms of Service",
                );
                return Ok((r, None, None));
            }
        }

        // ── phone verification via SMS ──
        if state.has_verification {
            random_jitter();
            // Try email verification first (poll temp_mail)
            state =
                Self::handle_email_verification(tab, provider, mailbox, auth_options.step_delay)?;
            if state.is_authenticated() {
                let r = result(
                    OpenAiAuthStage::Authenticated,
                    state,
                    "authenticated after post-password email verification",
                );
                return Ok((r, None, None));
            }

            if !state.has_verification {
                // Email verification resolved, try phone next
            }

            let (sms_phone, sms_order_id) =
                Self::handle_phone_verification(tab, reg_options, auth_options.step_delay)?;

            // Wait a moment for the final state
            state = wait_for_state(tab, Duration::from_secs(15), |s| {
                s.is_authenticated() || s.has_captcha
            })?;

            let r = if state.is_authenticated() {
                result(
                    OpenAiAuthStage::Authenticated,
                    state,
                    "authenticated after phone verification",
                )
            } else {
                result(
                    OpenAiAuthStage::VerificationRequired,
                    state,
                    "phone verification submitted; final state unknown",
                )
            };
            return Ok((r, sms_phone, sms_order_id));
        }

        // Fallback
        let r = result(
            OpenAiAuthStage::PasswordSubmitted,
            state,
            "flow ended without clear signal",
        );
        Ok((r, None, None))
    }

    /// Fills name and age on the "About You" onboarding page.
    /// Uses React Aria-compatible filling since these are likely
    /// `[role="textbox"]` elements, not native `<input>`.
    fn handle_onboarding(
        tab: &Arc<Tab>,
        _step_delay: Duration,
    ) -> BrowserAutomationResult<AuthPageState> {
        eprintln!("[DEBUG] handle_onboarding entered");
        // Fill name + age (first two visible textboxes in DOM order)
        let name = random_full_name();
        let age = random_age();
        eprintln!("[DEBUG] filling: name={} age={}", name, age);
        Self::fill_all_textboxes(tab, &[&name, &age.to_string()])?;
        thread::sleep(Duration::from_secs(2));

        // Click submit
        eprintln!("[DEBUG] attempting to click submit button...");
        let _ = click_button_by_text(
            tab,
            &[
                "Complete account creation",
                "Create account",
                "Finish",
                "Continue",
                "Next",
                "Submit",
                "完成帐户创建",
                "完成账户创建",
                "完成",
            ],
        );
        thread::sleep(Duration::from_secs(3));

        // Check for terms rejection before polling
        let immediate_state = read_state(tab)?;
        if immediate_state.has_terms_rejected {
            eprintln!("[DEBUG] handle_onboarding detected terms rejection");
            return Ok(immediate_state);
        }

        let state = wait_for_state(tab, Duration::from_secs(15), |s| {
            s.is_authenticated() || s.has_captcha || s.has_onboarding || s.has_terms_rejected
        })?;
        // If still on onboarding, return onboarding state
        if state.has_onboarding && !state.is_authenticated() {
            return Ok(state);
        }
        Ok(state)
    }

    /// Fills ALL visible textboxes on the page in DOM order.
    /// Uses execCommand for React Aria compatibility.
    fn fill_all_textboxes(tab: &Arc<Tab>, values: &[&str]) -> BrowserAutomationResult<bool> {
        let js_values = js_value(values)?;
        let script = format!(
            r#"
            (() => {{
                const values = {js_values};
                const textboxes = [
                    ...document.querySelectorAll('[role="textbox"]'),
                    ...document.querySelectorAll('input[type="text"]:not([readonly]):not([disabled])'),
                    ...document.querySelectorAll('input:not([type]):not([readonly]):not([disabled])'),
                    ...document.querySelectorAll('input[type="number"]:not([readonly]):not([disabled])'),
                ];
                const visible = (el) => {{
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    return !el.readOnly && !el.disabled
                        && style.visibility !== 'hidden'
                        && style.display !== 'none'
                        && rect.width > 0 && rect.height > 0;
                }};
                let filled = 0;
                for (const el of textboxes) {{
                    if (!visible(el) || filled >= values.length) continue;
                    const value = values[filled];
                    el.focus();
                    // Select all existing content
                    if (document.activeElement === el) {{
                        document.execCommand('selectAll', false, null);
                        // Type the value via execCommand (works for both input and contentEditable)
                        document.execCommand('insertText', false, value);
                    }}
                    // Also set value directly as fallback
                    if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {{
                        const nativeSetter = Object.getOwnPropertyDescriptor(
                            window.HTMLInputElement.prototype, 'value'
                        );
                        if (nativeSetter && nativeSetter.set) {{
                            nativeSetter.set.call(el, value);
                        }} else {{
                            el.value = value;
                        }}
                    }} else {{
                        el.textContent = value;
                    }}
                    // Dispatch React-compatible events
                    el.dispatchEvent(new InputEvent('beforeinput', {{
                        bubbles: true, cancelable: true,
                        data: value, inputType: 'insertText'
                    }}));
                    el.dispatchEvent(new InputEvent('input', {{
                        bubbles: true, data: value, inputType: 'insertText'
                    }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    el.dispatchEvent(new FocusEvent('blur', {{ bubbles: true }}));
                    filled++;
                }}
                return filled === values.length;
            }})()
            "#
        );
        let result = evaluate_json(tab, &script).map(|v| v.as_bool().unwrap_or(false));
        eprintln!("[DEBUG] fill_all_textboxes result={:?}", result);
        result
    }

    /// Fills a React Aria textbox by searching for a visible textbox whose
    /// accessible label matches one of the given labels.
    fn fill_react_textbox(
        tab: &Arc<Tab>,
        label_cn: &str,
        label_en: &str,
        value: &str,
    ) -> BrowserAutomationResult<bool> {
        let js_labels = js_value(&[label_cn, label_en])?;
        let js_value = js_value(value)?;
        let script = format!(
            r#"
            (() => {{
                const labels = {js_labels}.map(l => l.toLowerCase());
                const value = {js_value};
                // Find all textboxes (React Aria uses role="textbox")
                const textboxes = [
                    ...document.querySelectorAll('[role="textbox"]'),
                    ...document.querySelectorAll('input[type="text"]:not([readonly]):not([disabled])'),
                    ...document.querySelectorAll('input:not([type]):not([readonly]):not([disabled])'),
                ];
                const visible = (el) => {{
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    return style.visibility !== 'hidden'
                        && style.display !== 'none'
                        && rect.width > 0 && rect.height > 0;
                }};
                for (const el of textboxes) {{
                    if (!visible(el)) continue;
                    // Check accessible name
                    const ariaLabel = (el.getAttribute('aria-label') || '').toLowerCase();
                    const ariaLabelledby = el.getAttribute('aria-labelledby');
                    let labelText = ariaLabel;
                    if (ariaLabelledby) {{
                        const labelEl = document.getElementById(ariaLabelledby);
                        if (labelEl) labelText += ' ' + (labelEl.textContent || '').toLowerCase();
                    }}
                    // Also check nearby label elements
                    const prevLabel = el.closest('label')?.textContent?.toLowerCase() || '';
                    const matchesLabel = labels.some(l =>
                        labelText.includes(l) || prevLabel.includes(l)
                    );
                    if (!matchesLabel) continue;

                    // Found the right textbox — fill using React-compatible method
                    el.focus();
                    // For contentEditable divs
                    if (el.contentEditable === 'true' || el.isContentEditable) {{
                        el.textContent = '';
                        el.dispatchEvent(new InputEvent('beforeinput', {{
                            bubbles: true, inputType: 'insertText', data: value
                        }}));
                        document.execCommand('insertText', false, value);
                        el.dispatchEvent(new InputEvent('input', {{
                            bubbles: true, data: value, inputType: 'insertText'
                        }}));
                    }} else {{
                        // Native input — use React value setter
                        const nativeSetter = Object.getOwnPropertyDescriptor(
                            window.HTMLInputElement.prototype, 'value'
                        );
                        if (nativeSetter && nativeSetter.set) {{
                            nativeSetter.set.call(el, value);
                        }} else {{
                            el.value = value;
                        }}
                        el.dispatchEvent(new InputEvent('input', {{
                            bubbles: true, data: value, inputType: 'insertText'
                        }}));
                    }}
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                    return true;
                }}
                return false;
            }})()
            "#
        );
        let result = evaluate_json(tab, &script).map(|v| v.as_bool().unwrap_or(false));
        eprintln!("[DEBUG] fill_all_textboxes result={:?}", result);
        result
    }

    /// Polls the disposable mailbox for an email verification code and enters
    /// it into the page.
    fn handle_email_verification(
        tab: &Arc<Tab>,
        provider: &dyn TempMailProvider,
        mailbox: &TempMailMailbox,
        step_delay: Duration,
    ) -> BrowserAutomationResult<AuthPageState> {
        let code = Self::poll_temp_mail_code(provider, mailbox, Duration::from_secs(120));
        let Some(code) = code else {
            return read_state(tab);
        };

        // Find and fill verification code input
        Self::fill_verification_code(tab, &code)?;
        thread::sleep(step_delay);

        if !click_continue(tab)? {
            // Maybe the code was auto-submitted
            thread::sleep(Duration::from_secs(3));
        }

        wait_for_state(tab, Duration::from_secs(25), |s| {
            s.is_authenticated() || s.has_password_input || s.has_captcha
        })
    }

    /// Polls the temp mailbox for a verification code, looking for 4–8 digit
    /// numeric codes or common verification patterns.
    fn poll_temp_mail_code(
        provider: &dyn TempMailProvider,
        mailbox: &TempMailMailbox,
        max_wait: Duration,
    ) -> Option<String> {
        let deadline = Instant::now() + max_wait;
        let interval = Duration::from_secs(4);

        // Give the server at least a few seconds to deliver
        thread::sleep(Duration::from_secs(3));

        while Instant::now() < deadline {
            if let Ok(listing) = provider.list_messages(mailbox, PageRequest::new(10, 0)) {
                for summary in &listing.results {
                    if let Ok(Some(detail)) = provider.get_message(mailbox, &summary.id.to_string())
                    {
                        let combined =
                            format!("{} {} {}", summary.subject, detail.text, detail.html);
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

    /// Handles phone verification: buys a 5sim number, enters it, polls for
    /// the SMS code, and enters that.
    fn handle_phone_verification(
        tab: &Arc<Tab>,
        reg_options: &OpenAiFullRegOptions,
        step_delay: Duration,
    ) -> BrowserAutomationResult<(Option<String>, Option<u64>)> {
        let sms_token = match reg_options.sms_token.as_deref() {
            Some(t) => t,
            None => return Ok((None, None)),
        };

        // Buy SMS number via 5sim
        let (sms_phone, order_id) = Self::buy_sms_number(sms_token, reg_options)?;

        // Enter phone number into the page
        Self::fill_phone_input(tab, &sms_phone)?;
        thread::sleep(step_delay);

        if !click_continue(tab)? {
            thread::sleep(Duration::from_secs(3));
        }

        // Wait for the SMS verification input to appear
        thread::sleep(Duration::from_secs(3));

        // Poll 5sim for the SMS code
        let code = Self::poll_sms_code(sms_token, order_id, Duration::from_secs(180));
        if let Some(code) = code {
            Self::fill_verification_code(tab, &code)?;
            thread::sleep(step_delay);
            let _ = click_continue(tab);
            thread::sleep(Duration::from_secs(3));
        }

        Ok((Some(sms_phone), Some(order_id)))
    }

    /// Buys a one-time SMS activation number from 5sim.
    fn buy_sms_number(
        sms_token: &str,
        reg_options: &OpenAiFullRegOptions,
    ) -> BrowserAutomationResult<(String, u64)> {
        let rt = tokio::runtime::Runtime::new().map_err(to_browser_error)?;
        rt.block_on(async {
            let client = super::build_fivesim_provider(sms_token)?;
            let request = az_sms::model::SmsActivationRequest::new(
                &reg_options.sms_country,
                &reg_options.sms_operator,
                &reg_options.sms_product,
            )
            .map_err(to_browser_error)?;
            let order = client
                .buy_activation_number(request)
                .await
                .map_err(to_browser_error)?;

            Ok((order.phone, order.id))
        })
    }

    /// Polls 5sim for the SMS message containing a verification code.
    fn poll_sms_code(sms_token: &str, order_id: u64, max_wait: Duration) -> Option<String> {
        let rt = tokio::runtime::Runtime::new().ok()?;
        rt.block_on(async {
            let client = super::build_fivesim_provider(sms_token).ok()?;
            let options =
                az_sms::model::WaitForSmsOptions::new(max_wait, Duration::from_secs(5)).ok()?;
            match client.wait_for_sms(order_id, options).await {
                Ok(order) => {
                    // Prefer provider-extracted code
                    if let Some(code) = order.sms.first().and_then(|msg| msg.code.clone()) {
                        return Some(code);
                    }
                    // Fallback: extract code from text
                    order
                        .sms
                        .first()
                        .and_then(|msg| extract_verification_code(&msg.text))
                }
                Err(_) => None,
            }
        })
    }

    /// Fills a verification code into the first matching input on the page.
    fn fill_verification_code(tab: &Arc<Tab>, code: &str) -> BrowserAutomationResult<bool> {
        let selectors = js_value(VERIFICATION_CODE_SELECTORS)?;
        let value = js_value(code)?;
        let script = format!(
            r#"
            (() => {{
                const selectors = {selectors};
                const value = {value};
                const visible = (el) => {{
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    return !el.disabled && !el.readOnly
                        && style.visibility !== 'hidden'
                        && style.display !== 'none'
                        && rect.width > 0 && rect.height > 0;
                }};
                for (const selector of selectors) {{
                    for (const el of document.querySelectorAll(selector)) {{
                        if (!visible(el) || !('value' in el)) continue;
                        el.focus();
                        const desc = Object.getOwnPropertyDescriptor(
                            Object.getPrototypeOf(el), 'value');
                        if (desc && desc.set) {{ desc.set.call(el, value); }}
                        else {{ el.value = value; }}
                        el.dispatchEvent(new InputEvent('input', {{
                            bubbles: true, data: value, inputType: 'insertText'
                        }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                }}
                return false;
            }})()
            "#
        );
        let result = evaluate_json(tab, &script).map(|v| v.as_bool().unwrap_or(false));
        eprintln!("[DEBUG] fill_all_textboxes result={:?}", result);
        result
    }

    /// Fills a phone number into the first matching phone input.
    fn fill_phone_input(tab: &Arc<Tab>, phone: &str) -> BrowserAutomationResult<bool> {
        let selectors = js_value(PHONE_SELECTORS)?;
        let value = js_value(phone)?;
        let script = format!(
            r#"
            (() => {{
                const selectors = {selectors};
                const value = {value};
                const visible = (el) => {{
                    const style = window.getComputedStyle(el);
                    const rect = el.getBoundingClientRect();
                    return !el.disabled && !el.readOnly
                        && style.visibility !== 'hidden'
                        && style.display !== 'none'
                        && rect.width > 0 && rect.height > 0;
                }};
                for (const selector of selectors) {{
                    for (const el of document.querySelectorAll(selector)) {{
                        if (!visible(el) || !('value' in el)) continue;
                        el.focus();
                        const desc = Object.getOwnPropertyDescriptor(
                            Object.getPrototypeOf(el), 'value');
                        if (desc && desc.set) {{ desc.set.call(el, value); }}
                        else {{ el.value = value; }}
                        el.dispatchEvent(new InputEvent('input', {{
                            bubbles: true, data: value, inputType: 'insertText'
                        }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                }}
                return false;
            }})()
            "#
        );
        let result = evaluate_json(tab, &script).map(|v| v.as_bool().unwrap_or(false));
        eprintln!("[DEBUG] fill_all_textboxes result={:?}", result);
        result
    }
}

/// CSS selectors for verification code inputs.
const VERIFICATION_CODE_SELECTORS: &[&str] = &[
    "input[type='text'][maxlength='6']",
    "input[inputmode='numeric']",
    "input[name*='code' i]",
    "input[name*='otp' i]",
    "input[name*='verify' i]",
    "input[name*='verification' i]",
    "input[id*='code' i]",
    "input[id*='otp' i]",
    "input[id*='verify' i]",
    "input[placeholder*='code' i]",
    "input[placeholder*='6-digit' i]",
    "input[aria-label*='code' i]",
    "input[aria-label*='verification' i]",
];

/// CSS selectors for phone number inputs.
const PHONE_SELECTORS: &[&str] = &[
    "input[type='tel']",
    "input[name*='phone' i]",
    "input[name*='mobile' i]",
    "input[id*='phone' i]",
    "input[id*='mobile' i]",
    "input[autocomplete='tel']",
    "input[autocomplete='tel-national']",
    "input[placeholder*='phone' i]",
    "input[aria-label*='phone' i]",
];

/// Extracts a 4–8 digit verification code from a block of text.
fn extract_verification_code(text: &str) -> Option<String> {
    // Strip HTML tags first
    let plain = strip_html_tags(text);
    for pattern in &[
        r"(?i)(?:verification|security|confirmation|one-time|otp)\s*(?:code|number|pin)?\s*[:]?\s*(\d{4,8})",
        r"(?i)code\s*[:]?\s*(\d{4,8})",
        r"(?i)(\d{4,8})\s*(?:is your|is the)\s*(?:verification|security|auth)",
        r"\b(\d{6})\b",
        r"\b(\d{5})\b",
        r"\b(\d{4})\b",
    ] {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&plain) {
                return Some(caps[1].to_owned());
            }
        }
    }
    None
}

/// Removes HTML tags, keeping only the text content.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Generates a random ASCII string of the given length for passwords.
fn random_ascii_string(len: usize) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$";
    let mut state = seed as u64;
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            chars[(state >> 33) as usize % chars.len()] as char
        })
        .collect()
}

fn random_full_name() -> String {
    let first_names = [
        "James",
        "Mary",
        "Robert",
        "Patricia",
        "John",
        "Jennifer",
        "Michael",
        "Linda",
        "David",
        "Elizabeth",
        "William",
        "Barbara",
        "Richard",
        "Susan",
        "Joseph",
        "Jessica",
        "Thomas",
        "Sarah",
        "Christopher",
        "Karen",
        "Charles",
        "Lisa",
        "Daniel",
        "Nancy",
        "Matthew",
        "Betty",
        "Anthony",
        "Margaret",
        "Mark",
        "Sandra",
        "Donald",
        "Ashley",
        "Steven",
        "Dorothy",
        "Paul",
        "Kimberly",
        "Andrew",
        "Emily",
        "Joshua",
        "Donna",
        "Kenneth",
        "Michelle",
        "Kevin",
        "Carol",
        "Brian",
        "Amanda",
        "George",
        "Melissa",
        "Timothy",
        "Deborah",
        "Ronald",
        "Stephanie",
        "Edward",
        "Rebecca",
        "Jason",
        "Sharon",
        "Jeffrey",
        "Laura",
        "Ryan",
        "Cynthia",
        "Jacob",
        "Kathleen",
        "Gary",
        "Amy",
        "Nicholas",
        "Angela",
        "Eric",
        "Shirley",
        "Jonathan",
        "Anna",
        "Stephen",
        "Brenda",
        "Larry",
        "Pamela",
        "Justin",
        "Emma",
        "Scott",
        "Nicole",
        "Brandon",
        "Helen",
        "Benjamin",
        "Samantha",
        "Samuel",
        "Katherine",
        "Raymond",
        "Christine",
        "Gregory",
        "Debra",
        "Frank",
        "Rachel",
        "Alexander",
        "Carolyn",
        "Patrick",
        "Janet",
        "Jack",
        "Catherine",
        "Dennis",
        "Maria",
        "Jerry",
        "Heather",
        "Tyler",
        "Diane",
    ];
    let last_names = [
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Hernandez",
        "Lopez",
        "Gonzalez",
        "Wilson",
        "Anderson",
        "Thomas",
        "Taylor",
        "Moore",
        "Jackson",
        "Martin",
        "Lee",
        "Perez",
        "Thompson",
        "White",
        "Harris",
        "Sanchez",
        "Clark",
        "Ramirez",
        "Lewis",
        "Robinson",
        "Walker",
        "Young",
        "Allen",
        "King",
        "Wright",
        "Scott",
        "Torres",
        "Nguyen",
        "Hill",
        "Flores",
        "Green",
        "Adams",
        "Nelson",
        "Baker",
        "Hall",
        "Rivera",
        "Campbell",
        "Mitchell",
        "Carter",
        "Roberts",
        "Gomez",
        "Phillips",
        "Evans",
        "Turner",
        "Diaz",
        "Parker",
        "Cruz",
        "Edwards",
        "Collins",
        "Reyes",
        "Stewart",
        "Morris",
        "Morales",
        "Murphy",
        "Cook",
        "Rogers",
        "Gutierrez",
        "Ortiz",
        "Morgan",
        "Cooper",
        "Peterson",
        "Bailey",
        "Reed",
        "Kelly",
        "Howard",
        "Ramos",
        "Kim",
        "Cox",
        "Ward",
        "Richardson",
        "Watson",
        "Brooks",
        "Chavez",
        "Wood",
        "James",
        "Bennett",
        "Gray",
        "Mendoza",
        "Ruiz",
        "Hughes",
        "Price",
        "Alvarez",
        "Castillo",
    ];
    let mut state = random_seed();
    let fi = ((state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407))
        >> 33) as usize
        % first_names.len();
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let li = ((state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407))
        >> 33) as usize
        % last_names.len();
    format!("{} {}", first_names[fi], last_names[li])
}

fn random_age() -> u8 {
    let mut state = random_seed();
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Ages 18-55, weighted toward 22-40
    let raw = ((state >> 33) as usize) % 100;
    if raw < 50 {
        (22 + (raw % 19)) as u8
    } else if raw < 80 {
        (18 + (raw % 5)) as u8
    } else {
        (40 + (raw % 16)) as u8
    }
}

fn random_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn char_to_key_code(ch: char) -> String {
    match ch {
        'a'..='z' => format!("Key{}", ch.to_uppercase()),
        'A'..='Z' => format!("Key{ch}"),
        '0'..='9' => format!("Digit{ch}"),
        ' ' => "Space".into(),
        _ => format!("Key{ch}"),
    }
}

fn char_to_vk(ch: char) -> i32 {
    match ch {
        'a'..='z' => (ch as i32) - 32,
        'A'..='Z' => ch as i32,
        '0'..='9' => ch as i32,
        ' ' => 32,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_options_should_target_openai_login_page() {
        let options = OpenAiAuthOptions::login("owner@example.com");

        assert_eq!(options.flow, OpenAiAuthFlow::Login);
        assert_eq!(options.start_url, OpenAiAuthOptions::LOGIN_URL);
        assert_eq!(options.email.as_deref(), Some("owner@example.com"));
    }

    #[test]
    fn result_is_complete_only_when_authenticated() {
        let complete = OpenAiAuthResult {
            stage: OpenAiAuthStage::Authenticated,
            final_url: "https://platform.openai.com/".to_owned(),
            page_title: "OpenAI Platform".to_owned(),
            message: String::new(),
        };

        assert!(complete.is_complete());
    }

    #[test]
    fn recording_options_should_default_to_entry_sign_up_steps() {
        let options = OpenAiRecordingOptions::entry_sign_up();

        assert_eq!(
            options.steps,
            vec![
                OpenAiRecordingStep::OpenEntryPage,
                OpenAiRecordingStep::ClickLogin,
                OpenAiRecordingStep::ClickSignUp,
            ]
        );
    }

    #[test]
    fn recording_step_parser_should_accept_stable_step_ids() {
        assert_eq!(
            OpenAiRecordingStep::from_id("step2"),
            Some(OpenAiRecordingStep::ClickLogin)
        );
    }
}
