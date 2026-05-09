//! Trait-based building blocks for authorized registration workflows.

use crate::session::BrowserSession;
use crate::{BrowserAutomationError, BrowserAutomationResult};
use regex::Regex;
use std::path::PathBuf;

/// Workflow trait for a multi-step registration flow.
pub trait RegistrationFlow: Send + Sync {
    /// Returns the human-readable target service name.
    fn name(&self) -> &str;

    /// Returns the URL where the registration flow starts.
    fn start_url(&self) -> &str;

    /// Executes the complete registration flow with an existing email address.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserAutomationError`] when browser automation, provider
    /// polling, or flow-specific validation fails.
    fn execute(
        &self,
        session: &BrowserSession,
        email: &str,
    ) -> BrowserAutomationResult<RegistrationResult>;
}

/// Result returned by a [`RegistrationFlow`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationResult {
    /// Target service name.
    pub service: String,
    /// Email address used for the registration attempt.
    pub email: String,
    /// Whether the flow completed successfully.
    pub success: bool,
    /// Optional verification code observed during the flow.
    pub verification_code: Option<String>,
    /// Optional flow error message.
    pub error: Option<String>,
    /// Optional final or diagnostic screenshot path.
    pub screenshot_path: Option<PathBuf>,
}

impl RegistrationResult {
    /// Creates a successful registration result.
    #[must_use]
    pub fn success(service: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            email: email.into(),
            success: true,
            verification_code: None,
            error: None,
            screenshot_path: None,
        }
    }

    /// Creates a failed registration result.
    #[must_use]
    pub fn failure(
        service: impl Into<String>,
        email: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            service: service.into(),
            email: email.into(),
            success: false,
            verification_code: None,
            error: Some(error.into()),
            screenshot_path: None,
        }
    }

    /// Attaches a verification code to this result.
    #[must_use]
    pub fn with_verification_code(mut self, code: impl Into<String>) -> Self {
        self.verification_code = Some(code.into());
        self
    }

    /// Attaches a screenshot path to this result.
    #[must_use]
    pub fn with_screenshot(mut self, path: impl Into<PathBuf>) -> Self {
        self.screenshot_path = Some(path.into());
        self
    }
}

/// Extracts the first six-digit verification code from an email body.
#[must_use]
pub fn extract_verification_code(body: &str) -> Option<String> {
    let Ok(regex) = Regex::new(r"\b(\d{6})\b") else {
        return None;
    };
    regex
        .captures(body)
        .and_then(|captures| captures.get(1))
        .map(|code| code.as_str().to_owned())
}

/// Builds an explicit unsupported-flow error.
#[must_use]
pub fn unsupported_registration_flow(
    flow: impl Into<String>,
    reason: impl Into<String>,
) -> BrowserAutomationError {
    BrowserAutomationError::UnsupportedRegistrationFlow {
        flow: flow.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_verification_code_should_find_first_six_digit_token() {
        let code = extract_verification_code("Your code is 123456. Ref 99.");

        assert_eq!(code.as_deref(), Some("123456"));
    }

    #[test]
    fn extract_verification_code_should_ignore_non_six_digit_numbers() {
        let code = extract_verification_code("Ticket 12345 and order 1234567");

        assert!(code.is_none());
    }
}
