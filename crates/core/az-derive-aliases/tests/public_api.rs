use az_derive_aliases::{
    apply, deserialize_camel_clone_debug, deserialize_camel_eq, error_copy_eq,
    from_copy_eq_display, plain_code_default_enum, plain_code_enum, plain_copy_eq_hash,
    plain_copy_eq_hash_display, plain_copy_eq_hash_ord_display, plain_default_copy_eq,
    plain_default_copy_eq_display, plain_eq_hash_display, serde_camel_eq_default,
    serde_camel_partial_eq_default, serde_code_enum, serde_code_partial_eq, serde_eq_copy_display,
    serde_eq_hash_display, serde_kebab_code_enum, serde_kebab_eq, serde_lower_code_enum,
    serde_partial_eq_display, serde_upper_eq, serialize_camel_clone_debug, serialize_camel_eq,
};
use serde_json::Value;
use std::collections::HashSet;

macro_rules! nested_default_copy_eq {
    ($item:item) => {
        az_derive_aliases::plain_default_copy_eq! {
            $item
        }
    };
}

macro_rules! nested_default_copy_eq_display {
    ($item:item) => {
        az_derive_aliases::plain_default_copy_eq_display! {
            $item
        }
    };
}

#[apply(plain_default_copy_eq_display)]
struct FlagCode(u8);

#[apply(plain_default_copy_eq)]
struct DefaultCode(u8);

#[apply(plain_copy_eq_hash)]
struct HashCode(u8);

#[apply(plain_copy_eq_hash_ord_display)]
struct OrderedCode(u8);

#[apply(plain_eq_hash_display)]
struct HashDisplayCode(u8);

#[apply(plain_copy_eq_hash_display)]
struct CopyHashDisplayCode(u8);

#[apply(nested_default_copy_eq)]
struct NestedPlainCode(u8);

#[apply(nested_default_copy_eq_display)]
struct NestedCode(u8);

#[apply(plain_code_default_enum)]
enum DefaultMode {
    #[default]
    RoundRobin,
    LeastInFlight,
}

#[apply(plain_code_enum)]
enum CustomPlainCode {
    DefaultName,
    #[strum(serialize = "legacyCode")]
    LegacyCode,
}

#[apply(serde_code_enum)]
enum SnakeCode {
    ReadyNow,
}

#[apply(serde_kebab_code_enum)]
enum KebabCode {
    ReadyNow,
}

#[apply(serde_lower_code_enum)]
enum LowerCode {
    Ready,
}

#[apply(serialize_camel_eq)]
struct CamelWrite {
    request_id: String,
    retry_count: usize,
}

#[apply(serialize_camel_clone_debug)]
struct CamelDebugWrite {
    base_url: String,
    desktop_token: String,
}

#[apply(deserialize_camel_eq)]
struct CamelRead {
    response_code: String,
    payload_size: usize,
}

#[apply(deserialize_camel_clone_debug)]
struct CamelDebugRead {
    trace_id: String,
    retry_count: usize,
}

#[apply(serde_camel_eq_default)]
struct CamelDefaults {
    user_name: String,
    is_active: bool,
}

#[apply(serde_camel_partial_eq_default)]
struct CamelMetrics {
    load_ratio: f64,
}

#[apply(serde_kebab_eq)]
enum KebabExtensionPoint {
    ScriptEngine,
    UiContribution,
    Custom(String),
}

#[apply(serde_upper_eq)]
enum UpperOrderStatus {
    Pending,
    Received,
    #[serde(other)]
    Unknown,
}

#[apply(serde_code_partial_eq)]
enum SnakeConstraint {
    NotNull,
    Default(Value),
}

#[apply(serde_eq_hash_display)]
struct SerdeHashDisplayCode(u8);

#[apply(serde_eq_copy_display)]
struct SerdeCopyDisplayCode(u8);

#[apply(serde_partial_eq_display)]
struct SerdeDisplayRatio(f64);

#[apply(from_copy_eq_display)]
struct FromCopyDisplayCode(u8);

#[apply(error_copy_eq)]
#[error("copy error: {0}")]
struct CopyError(u8);

#[test]
fn layered_plain_aliases_keep_all_stacked_derives() {
    let flag = FlagCode(7);
    let copied = flag;

    assert_eq!(FlagCode::default(), FlagCode(0));
    assert_eq!(copied.to_string(), "7");

    assert_eq!(DefaultCode::default(), DefaultCode(0));

    let mut values = vec![OrderedCode(2), OrderedCode(1)];
    values.sort();
    assert_eq!(values, vec![OrderedCode(1), OrderedCode(2)]);

    let mut seen = HashSet::new();
    seen.insert(HashCode(1));
    assert!(seen.contains(&HashCode(1)));
}

#[test]
fn display_aliases_should_preserve_functional_layers() {
    let mut seen = HashSet::new();
    seen.insert(HashDisplayCode(7));

    let copied = CopyHashDisplayCode(8);

    assert!(seen.contains(&HashDisplayCode(7)));
    assert_eq!(HashDisplayCode(7).to_string(), "7");
    assert_eq!(copied.to_string(), "8");
}

#[test]
fn macro_rules_attribute_supports_alias_cascade() {
    assert_eq!(NestedPlainCode::default(), NestedPlainCode(0));

    let nested = NestedCode::default();

    assert_eq!(nested, NestedCode(0));
    assert_eq!(NestedCode(3).to_string(), "3");
}

#[test]
fn plain_code_default_enum_cascades_code_helpers_and_default() {
    assert_eq!(DefaultMode::default(), DefaultMode::RoundRobin);
    assert_eq!(DefaultMode::LeastInFlight.code(), "least_in_flight");
    assert_eq!(
        DefaultMode::from_code("round_robin"),
        Some(DefaultMode::RoundRobin)
    );
    assert_eq!(
        DefaultMode::from_code_or_default("unknown"),
        DefaultMode::RoundRobin
    );
}

#[test]
fn plain_code_enum_defaults_to_snake_case_and_allows_variant_overrides() {
    assert_eq!(CustomPlainCode::DefaultName.code(), "default_name");
    assert_eq!(
        CustomPlainCode::from_code("legacyCode"),
        Some(CustomPlainCode::LegacyCode)
    );
}

#[test]
fn serde_code_enum_aliases_reuse_case_aware_helper() {
    assert_eq!(SnakeCode::ReadyNow.code(), "ready_now");
    assert_eq!(KebabCode::ReadyNow.code(), "ready-now");
    assert_eq!(LowerCode::Ready.code(), "ready");

    assert_eq!(
        serde_json::to_value(SnakeCode::ReadyNow).unwrap(),
        serde_json::json!("ready_now")
    );
    assert_eq!(
        serde_json::to_value(KebabCode::ReadyNow).unwrap(),
        serde_json::json!("ready-now")
    );
    assert_eq!(
        serde_json::to_value(LowerCode::Ready).unwrap(),
        serde_json::json!("ready")
    );
}

#[test]
fn camel_case_aliases_apply_serde_rename_all() {
    let encoded: Value = serde_json::to_value(CamelWrite {
        request_id: "req-1".to_owned(),
        retry_count: 3,
    })
    .unwrap();

    assert_eq!(encoded["requestId"], "req-1");
    assert_eq!(encoded["retryCount"], 3);
    assert!(encoded.get("request_id").is_none());

    let debug_write = CamelDebugWrite {
        base_url: "http://127.0.0.1:3000".to_owned(),
        desktop_token: "token".to_owned(),
    };
    let encoded_debug: Value = serde_json::to_value(debug_write.clone()).unwrap();
    assert_eq!(encoded_debug["baseUrl"], "http://127.0.0.1:3000");
    assert_eq!(encoded_debug["desktopToken"], "token");
    assert!(format!("{debug_write:?}").contains("base_url"));

    let decoded: CamelRead =
        serde_json::from_str(r#"{"responseCode":"ok","payloadSize":4}"#).unwrap();
    assert_eq!(
        decoded,
        CamelRead {
            response_code: "ok".to_owned(),
            payload_size: 4,
        }
    );

    let debug_read: CamelDebugRead =
        serde_json::from_str(r#"{"traceId":"trace-1","retryCount":2}"#).unwrap();
    assert_eq!(debug_read.trace_id, "trace-1");
    assert_eq!(debug_read.retry_count, 2);
    assert_eq!(
        format!("{debug_read:?}"),
        format!("{:?}", debug_read.clone())
    );

    let defaults = serde_json::to_value(CamelDefaults::default()).unwrap();
    assert_eq!(defaults["userName"], "");
    assert_eq!(defaults["isActive"], false);
    assert_eq!(
        CamelDefaults::default(),
        CamelDefaults {
            user_name: String::new(),
            is_active: false,
        }
    );

    let metrics = serde_json::to_value(CamelMetrics::default()).unwrap();
    assert_eq!(metrics["loadRatio"], 0.0);
    assert_eq!(CamelMetrics::default(), CamelMetrics { load_ratio: 0.0 });
}

#[test]
fn kebab_case_aliases_apply_serde_rename_all() {
    let encoded = serde_json::to_value(KebabExtensionPoint::ScriptEngine).unwrap();
    assert_eq!(encoded, serde_json::json!("script-engine"));

    let decoded: KebabExtensionPoint = serde_json::from_str("\"ui-contribution\"").unwrap();
    assert_eq!(decoded, KebabExtensionPoint::UiContribution);

    let custom = KebabExtensionPoint::Custom("x".to_owned());
    assert_eq!(custom, KebabExtensionPoint::Custom("x".to_owned()));
}

#[test]
fn upper_and_snake_case_aliases_apply_serde_rename_all() {
    let encoded = serde_json::to_value(UpperOrderStatus::Pending).unwrap();
    assert_eq!(encoded, serde_json::json!("PENDING"));

    let decoded: UpperOrderStatus = serde_json::from_str("\"RECEIVED\"").unwrap();
    assert_eq!(decoded, UpperOrderStatus::Received);

    let unknown: UpperOrderStatus = serde_json::from_str("\"EXPIRED\"").unwrap();
    assert_eq!(unknown, UpperOrderStatus::Unknown);

    let snake = serde_json::to_value(SnakeConstraint::NotNull).unwrap();
    assert_eq!(snake, serde_json::json!("not_null"));
    let with_value = SnakeConstraint::Default(serde_json::json!(1));
    assert_eq!(
        serde_json::to_value(with_value).unwrap(),
        serde_json::json!({"default": 1})
    );
}

#[test]
fn serde_and_from_display_aliases_should_chain_cleanly() {
    let serde_hash = SerdeHashDisplayCode(9);
    let serde_copy = SerdeCopyDisplayCode(10);
    let from_copy: FromCopyDisplayCode = 11u8.into();
    let copy_error = CopyError(12);

    let encoded = serde_json::to_value(&serde_hash).unwrap();
    assert_eq!(encoded, serde_json::json!(9));
    assert_eq!(serde_hash.to_string(), "9");
    assert_eq!(serde_copy.to_string(), "10");
    assert_eq!(SerdeDisplayRatio(0.5).to_string(), "0.5");
    assert_eq!(from_copy.to_string(), "11");
    assert_eq!(copy_error.to_string(), "copy error: 12");
}
