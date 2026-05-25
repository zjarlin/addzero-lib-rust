use std::time::Duration;

use crate::error::{ProviderStatus, SmsError, SmsResult};

pub(crate) fn ensure_non_blank(field: &'static str, value: &str) -> SmsResult<()> {
    if value.trim().is_empty() {
        return Err(SmsError::InvalidConfig(format!("{field} cannot be blank")));
    }
    Ok(())
}

pub(crate) fn ensure_non_zero_duration(field: &'static str, value: Duration) -> SmsResult<()> {
    if value.is_zero() {
        return Err(SmsError::InvalidConfig(format!("{field} cannot be zero")));
    }
    Ok(())
}

pub(crate) fn build_client(
    connect_timeout: Duration,
    request_timeout: Duration,
    user_agent: Option<String>,
) -> SmsResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(request_timeout);
    if let Some(user_agent) = user_agent {
        builder = builder.user_agent(user_agent);
    }
    Ok(builder.build()?)
}

pub(crate) fn provider_error(status: Option<u16>, body: impl AsRef<str>) -> SmsError {
    SmsError::ProviderError {
        status: ProviderStatus(status),
        message: body.as_ref().trim().to_owned(),
    }
}

pub(crate) fn looks_like_provider_message(body: &str) -> bool {
    let trimmed = body.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('{')
        && !trimmed.starts_with('[')
        && trimmed.len() <= 256
}

pub(crate) fn default_user_agent() -> String {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).to_owned()
}
