use az_sms::error::{ProviderStatus, SmsError};

#[test]
fn provider_status_formats_only_when_present() {
    let err = SmsError::ProviderError {
        status: ProviderStatus::from(Some(400)),
        message: "bad country".to_owned(),
    };
    assert_eq!(err.to_string(), "provider error HTTP 400: bad country");

    let err = SmsError::ProviderError {
        status: ProviderStatus::from(None),
        message: "no free phones".to_owned(),
    };
    assert_eq!(err.to_string(), "provider error: no free phones");
}
