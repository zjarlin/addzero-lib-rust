//! Automation adapters for AI provider authorization screens.
//!
//! These helpers are intended for owner-authorized browser sessions. Provider
//! challenges such as CAPTCHA, MFA, and email verification are reported as
//! manual-action stages instead of being bypassed.

use crate::browser_automation::{BrowserAutomationError, BrowserAutomationResult};
use az_sms::model::{SmsActivationRequest, WaitForSmsOptions};
use az_temp_mail::{CreateMailboxRequest, TempMailMailbox, TempMailProvider};
use std::time::Duration;

pub mod gpt {
    pub mod api_reg;
    pub mod openai;
}

pub mod kiro {
    pub mod kiro;
}

pub use gpt::api_reg;
pub use gpt::openai;

pub use crate::ai_reg_auto::gpt::openai::{
    OpenAiAuthAutomation, OpenAiAuthFlow, OpenAiAuthOptions, OpenAiAuthResult, OpenAiAuthStage,
    OpenAiFullRegOptions, OpenAiFullRegResult, OpenAiRecordingOptions, OpenAiRecordingResult,
    OpenAiRecordingStep, OpenAiRecordingStepResult, OpenAiRecordingStepStatus, OpenAiRegAutomation,
};
pub use crate::ai_reg_auto::kiro::kiro::KiroRegistrationFlow;

pub fn build_fivesim_provider_with(
    factory: &dyn az_sms::provider::SmsProviderFactory,
    token: &str,
) -> crate::browser_automation::BrowserAutomationResult<az_sms::provider::BoxSmsProvider> {
    az_sms::fivesim_factory::build_fivesim_provider_with(factory, token).map_err(|error| {
        crate::browser_automation::BrowserAutomationError::Browser(error.to_string())
    })
}

pub fn build_fivesim_provider(
    token: &str,
) -> crate::browser_automation::BrowserAutomationResult<az_sms::provider::BoxSmsProvider> {
    az_sms::fivesim_factory::build_fivesim_provider(token).map_err(|error| {
        crate::browser_automation::BrowserAutomationError::Browser(error.to_string())
    })
}

pub(crate) fn create_registration_mailbox(
    provider: &dyn TempMailProvider,
    email_prefix: &str,
) -> BrowserAutomationResult<TempMailMailbox> {
    provider
        .create_mailbox(&CreateMailboxRequest::named(email_prefix).password_length(16))
        .map_err(|error| BrowserAutomationError::Browser(error.to_string()))
}

pub(crate) fn buy_fivesim_number_with(
    factory: &dyn az_sms::provider::SmsProviderFactory,
    sms_token: &str,
    country: &str,
    operator: &str,
    product: &str,
) -> BrowserAutomationResult<(String, u64)> {
    let rt = tokio::runtime::Runtime::new().map_err(|error| {
        BrowserAutomationError::Browser(format!("failed to initialize async runtime: {error}"))
    })?;
    rt.block_on(async {
        let client = build_fivesim_provider_with(factory, sms_token)?;
        let request = SmsActivationRequest::new(country, operator, product)
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        let order = client
            .buy_activation_number(request)
            .await
            .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
        Ok((order.phone, order.id))
    })
}

pub(crate) fn poll_fivesim_sms_with(
    factory: &dyn az_sms::provider::SmsProviderFactory,
    sms_token: &str,
    order_id: u64,
    max_wait: Duration,
) -> Option<String> {
    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        let client = build_fivesim_provider_with(factory, sms_token).ok()?;
        let options = WaitForSmsOptions::new(max_wait, Duration::from_secs(5)).ok()?;
        match client.wait_for_sms(order_id, options).await {
            Ok(order) => {
                if let Some(code) = order.sms.first().and_then(|msg| msg.code.clone()) {
                    return Some(code);
                }
                order
                    .sms
                    .first()
                    .and_then(|msg| extract_sms_code(&msg.text))
            }
            Err(_) => None,
        }
    })
}

fn extract_sms_code(text: &str) -> Option<String> {
    for pattern in &[
        r"(?i)(?:verification|security|confirmation|one-time|otp)\s*(?:code|number|pin)?\s*[:]?\s*(\d{4,8})",
        r"(?i)code\s*[:]?\s*(\d{4,8})",
        r"(?i)(\d{4,8})\s*(?:is your|is the)\s*(?:verification|security|auth)",
        r"\b(\d{6})\b",
        r"\b(\d{5})\b",
        r"\b(\d{4})\b",
    ] {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                return Some(caps[1].to_owned());
            }
        }
    }
    None
}
