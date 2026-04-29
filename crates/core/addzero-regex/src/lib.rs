//! Regex utilities with compiled pattern caching and helper functions.
//!
//! [`CachedRegex`] compiles a pattern once and reuses it for repeated matches,
//! while the free functions provide one-shot convenience helpers.

use std::collections::HashMap;

use regex::Captures;
use regex::Regex;

/// A pre-compiled regular expression that stores both the original pattern
/// string and the compiled [`Regex`], avoiding re-compilation on every use.
#[derive(Debug, Clone)]
pub struct CachedRegex {
    /// The original pattern string.
    pattern: String,
    /// The compiled regular expression.
    re: Regex,
}

impl CachedRegex {
    /// Compile a new [`CachedRegex`] from a pattern string.
    ///
    /// # Errors
    ///
    /// Returns [`regex::Error`] if the pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    ///
    /// let re = CachedRegex::new(r"\d+").unwrap();
    /// assert!(re.is_match("foo 42 bar"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let re = Regex::new(pattern)?;
        Ok(Self {
            pattern: pattern.to_owned(),
            re,
        })
    }

    /// Returns `true` if the regex matches anywhere in `text`.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    ///
    /// let re = CachedRegex::new(r"hello").unwrap();
    /// assert!(re.is_match("say hello world"));
    /// assert!(!re.is_match("nothing here"));
    /// ```
    pub fn is_match(&self, text: &str) -> bool {
        self.re.is_match(text)
    }

    /// Returns the first match as a string slice, or `None` if there is no match.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    ///
    /// let re = CachedRegex::new(r"\d+").unwrap();
    /// assert_eq!(re.find("abc 123 def"), Some("123"));
    /// assert_eq!(re.find("no digits"), None);
    /// ```
    pub fn find<'h>(&self, text: &'h str) -> Option<&'h str> {
        self.re.find(text).map(|m| m.as_str())
    }

    /// Returns the capture groups for the first match, or `None` if there is
    /// no match.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    ///
    /// let re = CachedRegex::new(r"(?<year>\d{4})-(?<month>\d{2})").unwrap();
    /// let caps = re.captures("2026-04").unwrap();
    /// assert_eq!(&caps["year"], "2026");
    /// ```
    pub fn captures<'h>(&self, text: &'h str) -> Option<Captures<'h>> {
        self.re.captures(text)
    }

    /// Replace all matches of the regex in `text` with a literal replacement string.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    ///
    /// let re = CachedRegex::new(r"\d+").unwrap();
    /// assert_eq!(re.replace_all("foo 1 bar 2", "#"), "foo # bar #");
    /// ```
    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        self.re.replace_all(text, replacement).into_owned()
    }

    /// Replace all matches of the regex in `text` using a closure that receives
    /// each set of captures and returns the replacement string.
    ///
    /// # Examples
    ///
    /// ```
    /// use addzero_regex::CachedRegex;
    /// use regex::Captures;
    ///
    /// let re = CachedRegex::new(r"(\d+)").unwrap();
    /// let result = re.replace_all_fn("a 2 b 3", |caps: &Captures| {
    ///     let n: i32 = caps[1].parse().unwrap();
    ///     (n * 10).to_string()
    /// });
    /// assert_eq!(result, "a 20 b 30");
    /// ```
    pub fn replace_all_fn(&self, text: &str, f: impl Fn(&Captures) -> String) -> String {
        self.re.replace_all(text, f).into_owned()
    }

    /// Returns the original pattern string.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns a reference to the underlying compiled [`Regex`].
    #[must_use]
    pub fn regex(&self) -> &Regex {
        &self.re
    }
}

/// Extract the named capture groups from [`Captures`] into a [`HashMap`].
///
/// Requires the compiled [`Regex`] to enumerate group names.
/// Unnamed groups are silently skipped.
///
/// # Examples
///
/// ```
/// use addzero_regex::{CachedRegex, named_captures_to_map};
///
/// let re = CachedRegex::new(r"(?<host>[^:]+):(?<port>\d+)").unwrap();
/// let caps = re.captures("localhost:8080").unwrap();
/// let map = named_captures_to_map(re.regex(), &caps);
/// assert_eq!(map["host"], "localhost");
/// assert_eq!(map["port"], "8080");
/// ```
#[must_use]
pub fn named_captures_to_map(re: &Regex, caps: &Captures) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for name in re.capture_names().flatten() {
        if let Some(m) = caps.name(name) {
            map.insert(name.to_owned(), m.as_str().to_owned());
        }
    }
    map
}

/// Extract all non-overlapping matches of `pattern` in `text` as owned strings.
///
/// # Errors
///
/// Returns an empty vector if the pattern is invalid.
///
/// # Examples
///
/// ```
/// use addzero_regex::extract_all;
///
/// let results = extract_all("foo 1 bar 23 baz 456", r"\d+");
/// assert_eq!(results, vec!["1", "23", "456"]);
/// ```
#[must_use]
pub fn extract_all(text: &str, pattern: &str) -> Vec<String> {
    let Ok(re) = Regex::new(pattern) else {
        return Vec::new();
    };
    re.find_iter(text).map(|m| m.as_str().to_owned()).collect()
}

/// Check whether `pattern` is a valid regular expression.
///
/// Returns `true` if it compiles, `false` otherwise.
///
/// # Examples
///
/// ```
/// use addzero_regex::is_valid_pattern;
///
/// assert!(is_valid_pattern(r"\d+"));
/// assert!(!is_valid_pattern("[invalid"));
/// ```
#[must_use]
pub fn is_valid_pattern(pattern: &str) -> bool {
    Regex::new(pattern).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_regex_is_match() {
        let re = CachedRegex::new(r"hello").unwrap();
        assert!(re.is_match("say hello world"));
        assert!(!re.is_match("nothing here"));
    }

    #[test]
    fn cached_regex_find() {
        let re = CachedRegex::new(r"\d+").unwrap();
        assert_eq!(re.find("abc 123 def"), Some("123"));
        assert_eq!(re.find("no digits here"), None);
    }

    #[test]
    fn cached_regex_captures_with_named_groups() {
        let re = CachedRegex::new(r"(?<year>\d{4})-(?<month>\d{2})-(?<day>\d{2})").unwrap();
        let caps = re.captures("today is 2026-04-29").unwrap();
        assert_eq!(&caps["year"], "2026");
        assert_eq!(&caps["month"], "04");
        assert_eq!(&caps["day"], "29");
    }

    #[test]
    fn cached_regex_replace_all() {
        let re = CachedRegex::new(r"\d+").unwrap();
        assert_eq!(re.replace_all("a1b2c3", "#"), "a#b#c#");
    }

    #[test]
    fn cached_regex_replace_all_fn() {
        let re = CachedRegex::new(r"(\d+)").unwrap();
        let result = re.replace_all_fn("a 2 b 3", |caps: &Captures| {
            let n: i32 = caps[1].parse().unwrap();
            (n * 10).to_string()
        });
        assert_eq!(result, "a 20 b 30");
    }

    #[test]
    fn named_captures_to_map_extracts_groups() {
        let re = CachedRegex::new(r"(?<host>[^:]+):(?<port>\d+)").unwrap();
        let caps = re.captures("localhost:8080").unwrap();
        let map = named_captures_to_map(re.regex(), &caps);
        assert_eq!(map["host"], "localhost");
        assert_eq!(map["port"], "8080");
    }

    #[test]
    fn extract_all_returns_all_matches() {
        let results = extract_all("foo 1 bar 23 baz 456", r"\d+");
        assert_eq!(results, vec!["1", "23", "456"]);
    }

    #[test]
    fn extract_all_returns_empty_for_invalid_pattern() {
        let results = extract_all("text", "[invalid");
        assert!(results.is_empty());
    }

    #[test]
    fn is_valid_pattern_returns_true_for_valid() {
        assert!(is_valid_pattern(r"\d+"));
        assert!(is_valid_pattern(r"(?<year>\d{4})"));
    }

    #[test]
    fn is_valid_pattern_returns_false_for_invalid() {
        assert!(!is_valid_pattern("[invalid"));
        assert!(!is_valid_pattern("(?<unclosed"));
    }
}
