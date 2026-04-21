use addzero_str::*;
use tempfile::TempDir;

#[test]
fn clean_blank_normalizes_whitespace_and_invisible_chars() {
    let value = clean_blank(Some("  hello\t\t世界\u{0001}\n "));
    assert_eq!(value, "hello 世界");
}

#[test]
fn default_table_name_prefers_existing_name_and_sanitizes_it() {
    let result = default_table_english_name("user_profile(test)", Some("用户信息"));
    assert_eq!(result, "user_profile");
}

#[test]
fn default_table_name_transliterates_when_english_name_is_blank() {
    let result = default_table_english_name("", Some("用户(表)"));
    assert!(!result.is_empty());
    assert!(!result.contains('('));
    assert!(
        result
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    );
}

#[test]
fn parent_path_and_mkdir_creates_child_directory() {
    let temp = TempDir::new().expect("temp dir should exist");
    let file_path = temp.path().join("logs/app.log");

    let created = parent_path_and_mkdir(&file_path, "archive").expect("directory should exist");

    assert_eq!(created, temp.path().join("logs/archive"));
    assert!(created.is_dir());
}

#[test]
fn trait_extension_for_parent_path_works_on_str() {
    let temp = TempDir::new().expect("temp dir should exist");
    let file_path = temp.path().join("reports/export.txt");
    let created = file_path
        .to_string_lossy()
        .as_ref()
        .parent_path_and_mkdir("history")
        .expect("directory should be created");

    assert_eq!(created, temp.path().join("reports/history"));
}

#[test]
fn naming_conversions_cover_camel_pascal_snake_and_kebab() {
    assert_eq!(to_camel_case("sys_yes_no", "", ""), "sysYesNo");
    assert_eq!(to_pascal_case("propSource", "", ""), "PropSource");
    assert_eq!(to_snake_case("XMLHttpRequest", "", ""), "xml_http_request");
    assert_eq!(
        to_kebab_name("hello_world-test", "", ""),
        "hello-world-test"
    );
    assert_eq!(to_constant_name("max value", "", ""), "MAX_VALUE");
}

#[test]
fn to_valid_variable_name_handles_digits_prefix_and_suffix() {
    assert_eq!(
        to_valid_variable_name("123", VariableType::CamelCase, "", ""),
        "__123"
    );
    assert_eq!(
        to_valid_variable_name("user name", VariableType::CamelCase, "my", "dto"),
        "myUserNameDto"
    );
    assert_eq!(
        to_valid_variable_name("order item", VariableType::SnakeCase, "erp", "entity"),
        "erp_order_item_entity"
    );
}

#[test]
fn underline_and_kebab_case_use_identifier_boundaries() {
    assert_eq!(to_underline_case("userName"), "user_Name");
    assert_eq!(
        to_underline_lower_case("XMLHttpRequest"),
        "xml_http_request"
    );
    assert_eq!(
        to_kebab_case("hello_world TestCase"),
        "hello-world-test-case"
    );
}

#[test]
fn prefix_suffix_and_remove_helpers_work() {
    assert_eq!(
        add_prefix_if_not(Some("world"), "hello ", false),
        "hello world"
    );
    assert_eq!(add_suffix_if_not(Some("file"), ".txt", false), "file.txt");
    assert_eq!(remove_any(Some(r#" "ab\c" "#), [" ", "\"", "\\"]), "abc");
    assert_eq!(
        remove_duplicate_symbol(Some("a----b"), Some("-")),
        Some("a-b".to_owned())
    );
}

#[test]
fn kmp_search_and_replace_work() {
    let matcher = KmpMatcher::new("aba");
    assert_eq!(matcher.search("xxabaxx"), 2);
    assert_eq!(matcher.search_all("ababa"), vec![0, 2]);
    assert!(contains_kmp("hello world", "world"));
    assert_eq!(replace_kmp("ababa", "aba", "X"), "Xba");
}

#[test]
fn extraction_helpers_work() {
    let pairs = extract_key_value_pairs("姓名：张三 年龄：25 city:Beijing");
    assert_eq!(pairs.get("姓名"), Some(&"张三".to_owned()));
    assert_eq!(pairs.get("年龄"), Some(&"25".to_owned()));
    assert_eq!(pairs.get("city"), Some(&"Beijing".to_owned()));

    assert_eq!(
        extract_text_between_p_tags(Some("<p>a</p><div>x</div><p>b</p>")),
        vec!["a".to_owned(), "b".to_owned()]
    );
    assert_eq!(
        get_rest_url(Some("http://localhost:8080/api/users/list")),
        "/users/list"
    );
}

#[test]
fn markdown_and_code_block_extractors_work() {
    let markdown = "before\n```json\n{\"name\":\"addzero\"}\n```\nafter";
    assert_eq!(
        extract_markdown_block_content(Some(markdown)),
        "{\"name\":\"addzero\"}"
    );
    assert_eq!(
        extract_markdown_block_content(Some("plain text")),
        "plain text"
    );
    assert_eq!(
        extract_code_block_content("``sql\nselect * from users\n``"),
        "select * from users"
    );
}

#[test]
fn doc_comment_and_blank_helpers_work() {
    let cleaned = clean_doc_comment(Some("/**\n * hello world\n */"));
    assert_eq!(cleaned, "hello world");
    assert_eq!(first_not_blank(&[None, Some(""), Some("  ok  ")]), "  ok  ");
    assert!(contains_chinese(Some("hello世界")));
    assert!(contains_any_ignore_case("HelloWorld", ["world", "test"]));
}

#[test]
fn format_helpers_work() {
    let value = format_template(
        "Name: %s, Age: %d, Score: %.1f, Hex: %x, Done: %%",
        &["John".into(), 30.into(), 95.5.into(), 255.into()],
    );
    assert_eq!(value, "Name: John, Age: 30, Score: 95.5, Hex: ff, Done: %");
    assert_eq!(kmp_format("Value: %.2f", &[3.14159.into()]), "Value: 3.14");
    assert_eq!(format_currency(19.99, 2), "19.99");
}

#[test]
fn text_and_misc_helpers_work() {
    assert_eq!(remove_not_chinese(Some("abc中文123")), "中文");
    assert_eq!(get_path_from_right("a.b.c", 1), "a.b");
    assert_eq!(lower_first("UserName"), "userName");
    assert_eq!(to_simple_name("site.addzero.UserName"), "UserName");
    assert_eq!(escape_special_characters("<a&b>"), "&lt;a&amp;b&gt;");
    assert!(is_number("-12.5"));
}
