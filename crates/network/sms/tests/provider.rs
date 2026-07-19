use az_sms::dogsms::client::DogSmsConfig;
use az_sms::grizzlysms::client::GrizzlySmsConfig;
use az_sms::provider::{
    BuiltinSmsProviderFactory, SmsProviderConfig, SmsProviderFactory, SmsProviderKind,
    build_sms_provider,
};

#[test]
fn provider_kind_uses_stable_wire_codes() {
    assert_eq!(SmsProviderKind::DogSms.code(), "dogsms");
    assert_eq!(SmsProviderKind::GrizzlySms.code(), "grizzly_sms");
    assert_eq!(
        SmsProviderKind::from_code("dogsms"),
        Some(SmsProviderKind::DogSms)
    );
}

#[test]
fn provider_config_reports_its_kind() {
    let config = DogSmsConfig::builder("key").build().unwrap();
    assert_eq!(
        SmsProviderConfig::from(config).kind(),
        SmsProviderKind::DogSms
    );

    let config = GrizzlySmsConfig::builder("key").build().unwrap();
    assert_eq!(
        SmsProviderConfig::from(config).kind(),
        SmsProviderKind::GrizzlySms
    );
}

#[test]
fn factory_builds_trait_objects_from_provider_configs() {
    let dogsms = DogSmsConfig::builder("key").build().unwrap();
    build_sms_provider(dogsms.into()).expect("dogsms provider should build");

    let grizzly = GrizzlySmsConfig::builder("key").build().unwrap();
    build_sms_provider(grizzly.into()).expect("grizzly sms provider should build");
}

#[test]
fn factory_trait_supports_dependency_injection() {
    let factory: &dyn SmsProviderFactory = &BuiltinSmsProviderFactory;
    let config = DogSmsConfig::builder("key").build().unwrap();

    factory
        .build_provider(config.into())
        .expect("injected provider factory should build");
}
