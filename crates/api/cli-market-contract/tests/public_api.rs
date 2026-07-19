use az_cli_market_contract::api::{CliInstallerKind, CliLocale, CliPlatform};

#[test]
fn code_enums_keep_public_wire_values() {
    assert_eq!(CliLocale::ZhCn.code(), "zh-CN");
    assert_eq!(CliLocale::from_code("en-US"), Some(CliLocale::EnUs));
    assert_eq!(
        serde_json::to_string(&CliLocale::ZhCn).expect("locale should serialize"),
        "\"zh-CN\""
    );

    assert_eq!(CliPlatform::CrossPlatform.code(), "cross_platform");
    assert_eq!(
        CliInstallerKind::from_code("brew"),
        Some(CliInstallerKind::Brew)
    );
}
