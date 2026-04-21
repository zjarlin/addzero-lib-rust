use addzero_pinyin::*;

#[test]
fn sanitize_creates_uppercase_identifier() {
    assert_eq!(sanitize("hello-world", "code"), "HELLO_WORLD");
    assert_eq!(sanitize("123", "code"), "__123");
    assert_eq!(sanitize("", "code"), "CODE");
}

#[test]
fn string_and_char_to_pinyin_convert_hanzi() {
    let chars = string_to_pinyin(Some("中国"), false, Some(",")).expect("should convert");

    assert_eq!(chars, vec!["zhong".to_owned(), "guo".to_owned()]);
    assert_eq!(char_to_pinyin('中', false, None), "zhong");
}

#[test]
fn polyphone_conversion_joins_all_readings() {
    let value = char_to_pinyin('还', true, Some(","));

    assert!(value.contains("hai"));
    assert!(value.contains(','));
}

#[test]
fn hanzi_to_pinyin_and_join_helpers_work() {
    assert_eq!(hanzi_to_pinyin("中国", Some("_")), "zhong_guo");
    assert_eq!(string_array_to_string(&["a", "b", "c"], Some("-")), "a-b-c");
    assert_eq!(char_array_to_string(&['A', 'B'], Some("")), "AB");
}

#[test]
fn head_helpers_return_initials() {
    let heads = get_head_by_char('中', true);
    assert!(!heads.is_empty());
    assert!(heads.iter().all(|head| *head == 'Z'));

    let words = get_head_by_string("中国", true, Some(""));
    assert_eq!(words[1], "G");
    assert!(words[0].chars().all(|head| head == 'Z'));
}
