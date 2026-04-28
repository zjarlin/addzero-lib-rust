use addzero_dict_macros::dict_enum;

dict_enum!(
    name = BoardParity,
    dict = "board_parity",
    spec = include_str!("../specs/int_open.json"),
    raw_type = u16
);

fn main() {
    assert_eq!(BoardParity::from_raw(0).label(), "None");
    assert_eq!(BoardParity::from_raw(2).meta_json(), Some("{\"legacy\":true}"));
    match BoardParity::from_raw(7) {
        BoardParity::Other(value) => assert_eq!(value, 7),
        _ => panic!("expected Other"),
    }
}
