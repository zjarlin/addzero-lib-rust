use addzero_dict_macros::dict_enum;

dict_enum!(
    name = BoardParity,
    dict = "board_parity",
    spec = include_str!("../specs/int_open.json"),
    raw_type = u16
);

fn main() {
    let item = BoardParity::items()
        .iter()
        .find(|entry| entry.code == "ODD")
        .expect("odd item should exist");
    assert_eq!(item.raw_value, 2);
    assert_eq!(item.meta_json, Some("{\"legacy\":true}"));
}
