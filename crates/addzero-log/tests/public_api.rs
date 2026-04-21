use addzero_log::*;

struct TestLogger;

#[test]
fn logger_target_uses_type_name_and_caches_it() {
    let first = logger_target::<TestLogger>();
    let second = logger_target::<TestLogger>();

    assert_eq!(first, second);
    assert!(first.ends_with("TestLogger"));
}

#[test]
fn value_logger_target_uses_runtime_value_type_name() {
    let logger = TestLogger;
    let target = value_logger_target(&logger);

    assert!(target.ends_with("TestLogger"));
}
