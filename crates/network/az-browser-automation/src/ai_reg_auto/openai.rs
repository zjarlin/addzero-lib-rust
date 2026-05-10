//! OpenAI login and sign-up page automation.

use crate::BrowserAutomationResult;
use crate::{BrowserAutomation, BrowserAutomationError, BrowserAutomationOptions};
use headless_chrome::Tab;
use headless_chrome::protocol::cdp::Runtime;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// OpenAI authorization flow mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAuthFlow {
    /// Open the login flow.
    Login,
    /// Open the sign-up flow.
    SignUp,
}

/// Current stage reached by [`OpenAiAuthAutomation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// Step identifiers for manual OpenAI entry-flow recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiRecordingStepStatus {
    /// The target for the step was found and clicked, or the page was ready.
    Completed,
    /// The browser stayed open but the expected target for the step was not found.
    MissingTarget,
}

/// Result for one recorded step.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Default, Clone, Copy)]
pub struct OpenAiAuthAutomation;

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

    fn run_on_tab(
        tab: &Arc<Tab>,
        auth_options: &OpenAiAuthOptions,
    ) -> BrowserAutomationResult<OpenAiAuthResult> {
        thread::sleep(auth_options.step_delay);

        if auth_options.flow == OpenAiAuthFlow::SignUp {
            let _ = click_button_by_text(tab, &["Sign up", "Create account"]);
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
    "input[autocomplete='username']",
    "input[placeholder*='email' i]",
    "input[aria-label*='email' i]",
];

const PASSWORD_SELECTORS: &[&str] = &[
    "input[type='password']",
    "input[name='password']",
    "input[autocomplete='current-password']",
    "input[autocomplete='new-password']",
    "input[placeholder*='password' i]",
    "input[aria-label*='password' i]",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthPageState {
    url: String,
    title: String,
    has_password_input: bool,
    has_verification: bool,
    has_captcha: bool,
}

impl AuthPageState {
    fn is_authenticated(&self) -> bool {
        let url = self.url.to_ascii_lowercase();
        url.contains("platform.openai.com")
            || url.contains("chatgpt.com")
            || url.contains("chat.openai.com")
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
            const hasVerification = [
                'verify',
                'verification',
                'check your email',
                'security code',
                'multi-factor',
                'two-factor',
                'authenticator',
                'one-time code'
            ].some((token) => bodyText.includes(token));
            const hasCaptcha = bodyText.includes('captcha')
                || document.querySelector('[class*="captcha" i], [id*="captcha" i], iframe[src*="captcha" i], iframe[src*="hcaptcha" i], iframe[src*="recaptcha" i], iframe[src*="turnstile" i]') !== null;
            return {
                url: window.location.href,
                title: document.title,
                hasPasswordInput,
                hasVerification,
                hasCaptcha
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
    evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false))
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
    evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false))
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
    evaluate_json(tab, &script).map(|value| value.as_bool().unwrap_or(false))
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
