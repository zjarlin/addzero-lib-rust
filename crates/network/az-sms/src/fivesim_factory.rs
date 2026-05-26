use crate::{
    error::SmsResult,
    fivesim::FivesimConfig,
    provider::{BoxSmsProvider, BuiltinSmsProviderFactory, SmsProviderConfig, SmsProviderFactory},
};

/// Build a 5sim provider through an injected factory.
pub fn build_fivesim_provider_with(
    factory: &dyn SmsProviderFactory,
    token: &str,
) -> SmsResult<BoxSmsProvider> {
    let config = FivesimConfig::builder(token).build()?;
    factory.build_provider(SmsProviderConfig::from(config))
}

/// Build the built-in 5sim provider through the shared SMS factory boundary.
pub fn build_fivesim_provider(token: &str) -> SmsResult<BoxSmsProvider> {
    build_fivesim_provider_with(&BuiltinSmsProviderFactory, token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::SmsProviderKind;
    use az_derive_aliases::{apply, plain_default};

    #[apply(plain_default)]
    struct InspectingFactory;

    impl SmsProviderFactory for InspectingFactory {
        fn build_provider(&self, config: SmsProviderConfig) -> SmsResult<BoxSmsProvider> {
            assert_eq!(config.kind(), SmsProviderKind::Fivesim);
            crate::provider::build_sms_provider(config)
        }
    }

    #[test]
    fn injected_factory_receives_fivesim_config() {
        let factory = InspectingFactory;
        let provider = build_fivesim_provider_with(&factory, "token").unwrap();
        drop(provider);
    }
}
