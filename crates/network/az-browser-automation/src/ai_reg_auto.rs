//! Automation adapters for AI provider authorization screens.
//!
//! These helpers are intended for owner-authorized browser sessions. Provider
//! challenges such as CAPTCHA, MFA, and email verification are reported as
//! manual-action stages instead of being bypassed.

pub mod gpt {
    pub mod api_reg;
    pub mod openai;
}

pub mod kiro {
    pub mod kiro;
}

pub mod sms;

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
