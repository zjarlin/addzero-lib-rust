use az_sms::fivesim::FivesimConfig;
use az_sms::grizzlysms::GrizzlySmsConfig;
use az_sms::provider::{
    BuiltinSmsProviderFactory, SmsProviderConfig, SmsProviderFactory, SmsProviderKind,
    build_sms_provider,
};

#[test]
fn provider_kind_uses_stable_wire_codes() {
    assert_eq!(SmsProviderKind::Fivesim.code(), "5sim");
    assert_eq!(SmsProviderKind::GrizzlySms.code(), "grizzly_sms");
    assert_eq!(
        SmsProviderKind::from_code("5sim"),
        Some(SmsProviderKind::Fivesim)
    );
}

#[test]
fn provider_config_reports_its_kind() {
    let config = FivesimConfig::builder("token").build().unwrap();
    assert_eq!(
        SmsProviderConfig::from(config).kind(),
        SmsProviderKind::Fivesim
    );

    let config = GrizzlySmsConfig::builder("key").build().unwrap();
    assert_eq!(
        SmsProviderConfig::from(config).kind(),
        SmsProviderKind::GrizzlySms
    );
}

#[test]
fn factory_builds_trait_objects_from_provider_configs() {
    let fivesim = FivesimConfig::builder("token").build().unwrap();
    build_sms_provider(fivesim.into()).expect("5sim provider should build");

    let grizzly = GrizzlySmsConfig::builder("key").build().unwrap();
    build_sms_provider(grizzly.into()).expect("grizzly sms provider should build");
}

#[test]
fn factory_trait_supports_dependency_injection() {
    let factory: &dyn SmsProviderFactory = &BuiltinSmsProviderFactory;
    let config = FivesimConfig::builder("token").build().unwrap();

    factory
        .build_provider(config.into())
        .expect("injected provider factory should build");
}
