use addzero_dict_macros::dict_enum;

dict_enum!(
    name = BoardTransportType,
    dict = "board_transport_type",
    spec = include_str!("../specs/string_closed.json")
);

fn main() {
    assert!(matches!(BoardTransportType::from_raw("RTU"), Some(BoardTransportType::Rtu)));
    assert!(BoardTransportType::from_raw("UDP").is_none());
}
