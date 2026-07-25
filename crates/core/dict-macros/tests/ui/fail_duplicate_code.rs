use az_dict_macros::dict_enum;

dict_enum!(
    name = DuplicateCode,
    dict = "duplicate_code",
    spec = include_str!("../specs/duplicate_code.json")
);

fn main() {}
