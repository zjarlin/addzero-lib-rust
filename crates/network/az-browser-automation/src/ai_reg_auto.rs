//! Automation adapters for AI provider authorization screens.
//!
//! These helpers are intended for owner-authorized browser sessions. Provider
//! challenges such as CAPTCHA, MFA, and email verification are reported as
//! manual-action stages instead of being bypassed.

pub mod api_reg;
pub mod openai;

pub use openai::{
    OpenAiAuthAutomation, OpenAiAuthFlow, OpenAiAuthOptions, OpenAiAuthResult, OpenAiAuthStage,
    OpenAiFullRegOptions, OpenAiFullRegResult, OpenAiRecordingOptions, OpenAiRecordingResult,
    OpenAiRecordingStep, OpenAiRecordingStepResult, OpenAiRecordingStepStatus, OpenAiRegAutomation,
};
