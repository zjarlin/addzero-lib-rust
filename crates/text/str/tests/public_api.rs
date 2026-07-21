use az_str::transformation::{
    FormatArg, KmpMatcher, MarkdownListMarkerMode, ParentPathExt, VariableType, add_prefix_if_not,
    add_suffix_if_not, clean_blank, clean_doc_comment, clean_markdown_plain_text,
    collapse_whitespace, contains_any_ignore_case, contains_chinese, contains_kmp,
    default_table_english_name, ensure_leading_slash, escape_special_characters,
    escape_sql_string_literal, escape_xml, extract_code_block_content, extract_key_value_pairs,
    extract_markdown_block_content, extract_text_between_p_tags, first_not_blank, format_currency,
    format_template, get_path_from_right, get_rest_url, is_number, kmp_format, lower_first,
    normalize_url_path, normalized_id_or_else, parent_path_and_mkdir, quote_posix_shell_single,
    quote_sql_string_literal, remove_any, remove_duplicate_symbol, remove_not_chinese, replace_kmp,
    split_url_path_segments, to_camel_case, to_constant_name, to_kebab_case, to_kebab_name,
    to_pascal_case, to_simple_name, to_snake_case, to_underline_case, to_underline_lower_case,
    to_valid_variable_name, trim_non_blank, trim_non_blank_owned, truncate_chars,
    truncate_chars_with_ellipsis, unescape_basic_html_entities,
};
use az_str::sanitize::{
    ascii_alphanumeric, ascii_alphanumeric_or, sanitize_ascii_label, sanitize_ascii_label_or,
    sanitize_file_name_with_extension, sanitize_file_stem, sanitize_file_stem_or,
    sanitize_path_file_stem_or, sanitize_path_segment, title_case_slug, to_slash_path, to_slug,
    to_slug_or,
};
use std::f64::consts::PI;
use std::path::Path;
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
fn variable_type_exposes_stable_codes() {
    assert_eq!(VariableType::CamelCase.code(), "camel_case");
    assert_eq!(
        VariableType::from_code("kebab_case"),
        Some(VariableType::KebabCase)
    );
    assert_eq!(VariableType::ALL.len(), 5);
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
    let preview = "# 标题\n- 第一行\n* 第二行\n```rust\nfn main() {}\n```";
    assert_eq!(
        clean_markdown_plain_text(preview, MarkdownListMarkerMode::Keep),
        "- 第一行 * 第二行"
    );
    assert_eq!(
        clean_markdown_plain_text(preview, MarkdownListMarkerMode::Strip),
        "第一行 第二行"
    );
}

#[test]
fn doc_comment_and_blank_helpers_work() {
    let cleaned = clean_doc_comment(Some("/**\n * hello world\n */"));
    assert_eq!(cleaned, "hello world");
    assert_eq!(first_not_blank(&[None, Some(""), Some("  ok  ")]), "  ok  ");
    assert!(contains_chinese(Some("hello世界")));
    assert!(contains_any_ignore_case("HelloWorld", ["world", "test"]));
    assert_eq!(trim_non_blank(Some("  ok  ")), Some("ok"));
    assert_eq!(trim_non_blank(Some("   ")), None);
    assert_eq!(
        trim_non_blank_owned(Some("  ok  ".to_owned())),
        Some("ok".to_owned())
    );
    assert_eq!(
        normalized_id_or_else(Some("  item-1  ".to_owned()), || "fallback".to_owned()),
        "item-1"
    );
    assert_eq!(
        normalized_id_or_else(Some("   ".to_owned()), || "fallback".to_owned()),
        "fallback"
    );
    assert_eq!(collapse_whitespace("  a\t\n b  "), "a b");
    assert_eq!(truncate_chars("你好世界", 2), "你好");
    assert_eq!(truncate_chars_with_ellipsis("你好世界", 2), "你好…");
    assert_eq!(truncate_chars_with_ellipsis("你好", 2), "你好");
}

#[test]
fn format_helpers_work() {
    let value = format_template(
        "Name: %s, Age: %d, Score: %.1f, Hex: %x, Done: %%",
        &["John".into(), 30.into(), 95.5.into(), 255.into()],
    );
    assert_eq!(value, "Name: John, Age: 30, Score: 95.5, Hex: ff, Done: %");
    assert_eq!(
        format_template("%s %s %s", &[FormatArg::Null, true.into(), 7usize.into()]),
        "null true 7"
    );
    assert_eq!(kmp_format("Value: %.2f", &[PI.into()]), "Value: 3.14");
    assert_eq!(format_currency(19.99, 2), "19.99");
}

#[test]
fn text_and_misc_helpers_work() {
    assert_eq!(remove_not_chinese(Some("abc中文123")), "中文");
    assert_eq!(get_path_from_right("a.b.c", 1), "a.b");
    assert_eq!(lower_first("UserName"), "userName");
    assert_eq!(to_simple_name("site.addzero.UserName"), "UserName");
    assert_eq!(escape_special_characters("<a&b>"), "&lt;a&amp;b&gt;");
    assert_eq!(
        escape_xml("<tag name=\"a'b&c\">"),
        "&lt;tag name=&quot;a&apos;b&amp;c&quot;&gt;"
    );
    assert_eq!(
        unescape_basic_html_entities("A &amp; B &quot;C&quot; &nbsp; &#39;"),
        "A & B \"C\"   '"
    );
    assert_eq!(escape_sql_string_literal("Bob's"), "Bob''s");
    assert_eq!(quote_sql_string_literal("Bob's"), "'Bob''s'");
    assert_eq!(quote_posix_shell_single("a'b"), "'a'\\''b'");
    assert_eq!(ensure_leading_slash(" callback "), "/callback");
    assert_eq!(normalize_url_path(" /api/users?id=1#top "), "/api/users");
    assert_eq!(
        split_url_path_segments("/api/users//42?debug=true"),
        vec!["api".to_owned(), "users".to_owned(), "42".to_owned()]
    );
    assert!(is_number("-12.5"));
}

#[test]
fn sanitize_helpers_cover_path_stem_slug_and_display_path() {
    assert_eq!(sanitize_path_segment(" a/b:C.mp4 "), "a-b-C.mp4");
    assert_eq!(
        sanitize_ascii_label("user+name@example.com", "@._-+", '_'),
        "user+name@example.com"
    );
    assert_eq!(sanitize_ascii_label_or("!!!", "-_", '-', "device"), "---");
    assert_eq!(sanitize_ascii_label_or("", "-_", '-', "device"), "device");
    assert_eq!(sanitize_file_stem("a/b:C.mp4"), "a_b_C_mp4");
    assert_eq!(sanitize_file_stem_or("", "input_video"), "input_video");
    assert_eq!(ascii_alphanumeric("az mail+01"), "azmail01");
    assert_eq!(ascii_alphanumeric_or("!!!", "az"), "az");
    assert_eq!(
        sanitize_path_file_stem_or(Path::new("a/b:C.mp4"), "input_video"),
        "b_C"
    );
    assert_eq!(
        sanitize_file_name_with_extension("../user@example.com", "@._-+", '_', "json", "auth"),
        ".._user@example.com.json"
    );
    assert_eq!(to_slug("Café Tool.md"), "cafe-tool-md");
    assert_eq!(to_slug_or("!!!", "doc"), "doc");
    assert_eq!(title_case_slug("docker-desktop.app"), "Docker Desktop App");
    assert_eq!(to_slash_path(Path::new("alpha/beta")), "alpha/beta");
    assert_eq!(to_slash_path(Path::new("/tmp/alpha")), "/tmp/alpha");
}
