use addzero_dict_macros::dict_enum;

dict_enum!(
    name = BoardParity,
    dict = "board_parity",
    spec = include_str!("specs/int_open.json"),
    raw_type = u16
);

dict_enum!(
    name = BoardTransportType,
    dict = "board_transport_type",
    spec = include_str!("specs/string_closed.json")
);

dict_enum!(
    name = BoardTransportTypeFromEnv,
    dict = "board_transport_type",
    spec = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/specs/string_closed.json"
    ))
);

#[test]
fn int_open_enum_supports_unknown_values() {
    assert_eq!(BoardParity::from_raw(0).code(), "NONE");
    assert_eq!(BoardParity::from_raw(0).raw_value(), 0);
    match BoardParity::from_raw(9) {
        BoardParity::Other(value) => assert_eq!(value, 9),
        _ => panic!("expected Other"),
    }
}

#[test]
fn string_closed_enum_supports_lookup() {
    assert!(matches!(
        BoardTransportType::from_raw("TCP"),
        Some(BoardTransportType::Tcp)
    ));
    assert_eq!(BoardTransportType::Tcp.label(), "TCP");
    assert_eq!(BoardTransportType::items().len(), 2);
    assert!(matches!(
        BoardTransportTypeFromEnv::from_raw("RTU"),
        Some(BoardTransportTypeFromEnv::Rtu)
    ));
}

#[test]
fn ui_compile_tests_pass() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/pass_int_open.rs");
    tests.pass("tests/ui/pass_string_closed.rs");
    tests.pass("tests/ui/pass_metadata_lookup.rs");
}
