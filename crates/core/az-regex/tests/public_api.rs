use az_regex::{CachedRegex, extract_all, is_valid_pattern, named_captures_to_map};
use regex::Captures;

#[test]
fn cached_regex_checks_matches_and_finds_first_match() {
    let word = CachedRegex::new(r"hello").unwrap();
    assert!(word.is_match("say hello world"));
    assert!(!word.is_match("nothing here"));

    let digits = CachedRegex::new(r"\d+").unwrap();
    assert_eq!(digits.find("abc 123 def"), Some("123"));
    assert_eq!(digits.find("no digits here"), None);
}

#[test]
fn cached_regex_exposes_captures_and_named_capture_map() {
    let date = CachedRegex::new(r"(?<year>\d{4})-(?<month>\d{2})-(?<day>\d{2})").unwrap();
    let caps = date.captures("today is 2026-04-29").unwrap();
    assert_eq!(&caps["year"], "2026");
    assert_eq!(&caps["month"], "04");
    assert_eq!(&caps["day"], "29");

    let host = CachedRegex::new(r"(?<host>[^:]+):(?<port>\d+)").unwrap();
    let caps = host.captures("localhost:8080").unwrap();
    let map = named_captures_to_map(host.regex(), &caps);
    assert_eq!(map["host"], "localhost");
    assert_eq!(map["port"], "8080");
}

#[test]
fn cached_regex_replaces_literals_and_closure_results() {
    let digits = CachedRegex::new(r"\d+").unwrap();
    assert_eq!(digits.replace_all("a1b2c3", "#"), "a#b#c#");

    let result = digits.replace_all_fn("a 2 b 3", |caps: &Captures| {
        let value: i32 = caps[0].parse().unwrap();
        (value * 10).to_string()
    });
    assert_eq!(result, "a 20 b 30");
}

#[test]
fn extract_all_returns_matches_or_empty_for_invalid_patterns() {
    let results = extract_all("foo 1 bar 23 baz 456", r"\d+");
    assert_eq!(results, vec!["1", "23", "456"]);

    let invalid = extract_all("text", "[invalid");
    assert!(invalid.is_empty());
}

#[test]
fn is_valid_pattern_reports_regex_compile_status() {
    assert!(is_valid_pattern(r"\d+"));
    assert!(is_valid_pattern(r"(?<year>\d{4})"));
    assert!(!is_valid_pattern("[invalid"));
    assert!(!is_valid_pattern("(?<unclosed"));
}
