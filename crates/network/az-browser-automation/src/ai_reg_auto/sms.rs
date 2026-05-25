use crate::{BrowserAutomationError, BrowserAutomationResult};
use az_sms::{
    fivesim::FivesimConfig,
    provider::{BoxSmsProvider, BuiltinSmsProviderFactory, SmsProviderConfig, SmsProviderFactory},
};

/// Build a provider through an injected factory.
pub fn build_fivesim_provider_with(
    factory: &dyn SmsProviderFactory,
    token: &str,
) -> BrowserAutomationResult<BoxSmsProvider> {
    let config = FivesimConfig::builder(token)
        .build()
        .map_err(|error| BrowserAutomationError::Browser(error.to_string()))?;
    factory
        .build_provider(SmsProviderConfig::from(config))
        .map_err(|error| BrowserAutomationError::Browser(error.to_string()))
}

/// Build the built-in 5sim provider through the shared SMS factory boundary.
pub fn build_fivesim_provider(token: &str) -> BrowserAutomationResult<BoxSmsProvider> {
    build_fivesim_provider_with(&BuiltinSmsProviderFactory, token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use az_derive_aliases::{apply, plain_default};
    use az_sms::provider::{BoxSmsProvider, SmsProviderKind};

    #[apply(plain_default)]
    struct InspectingFactory;

    impl SmsProviderFactory for InspectingFactory {
        fn build_provider(
            &self,
            config: SmsProviderConfig,
        ) -> az_sms::error::SmsResult<BoxSmsProvider> {
            assert_eq!(config.kind(), SmsProviderKind::Fivesim);
            az_sms::provider::build_sms_provider(config)
        }
    }

    #[test]
    fn injected_factory_receives_fivesim_config() {
        let factory = InspectingFactory;
        let provider = build_fivesim_provider_with(&factory, "token").unwrap();
        drop(provider);
    }
}
