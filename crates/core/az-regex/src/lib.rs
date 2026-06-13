//! 带编译模式缓存和辅助函数的正则表达式工具库。
//!
//! [`CachedRegex`] 编译一次正则模式并在重复匹配时复用，
//! 同时提供自由函数作为一次性便捷辅助。

use std::collections::HashMap;

use az_derive_aliases::{apply, plain_clone_debug};
use regex::Captures;
use regex::Regex;

/// A pre-compiled regular expression that stores both the original pattern
/// string and the compiled [`Regex`], avoiding re-compilation on every use.
#[apply(plain_clone_debug)]
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
    /// use az_regex::CachedRegex;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"\d+")?;
    /// assert!(re.is_match("foo 42 bar"));
    /// # Ok(())
    /// # }
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
    /// use az_regex::CachedRegex;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"hello")?;
    /// assert!(re.is_match("say hello world"));
    /// assert!(!re.is_match("nothing here"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_match(&self, text: &str) -> bool {
        self.re.is_match(text)
    }

    /// Returns the first match as a string slice, or `None` if there is no match.
    ///
    /// # Examples
    ///
    /// ```
    /// use az_regex::CachedRegex;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"\d+")?;
    /// assert_eq!(re.find("abc 123 def"), Some("123"));
    /// assert_eq!(re.find("no digits"), None);
    /// # Ok(())
    /// # }
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
    /// use az_regex::CachedRegex;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"(?<year>\d{4})-(?<month>\d{2})")?;
    /// let caps = re.captures("2026-04").ok_or("missing captures")?;
    /// assert_eq!(&caps["year"], "2026");
    /// # Ok(())
    /// # }
    /// ```
    pub fn captures<'h>(&self, text: &'h str) -> Option<Captures<'h>> {
        self.re.captures(text)
    }

    /// Replace all matches of the regex in `text` with a literal replacement string.
    ///
    /// # Examples
    ///
    /// ```
    /// use az_regex::CachedRegex;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"\d+")?;
    /// assert_eq!(re.replace_all("foo 1 bar 2", "#"), "foo # bar #");
    /// # Ok(())
    /// # }
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
    /// use az_regex::CachedRegex;
    /// use regex::Captures;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let re = CachedRegex::new(r"(\d+)")?;
    /// let result = re.replace_all_fn("a 2 b 3", |caps: &Captures| {
    ///     format!("[{}]", &caps[1])
    /// });
    /// assert_eq!(result, "a [2] b [3]");
    /// # Ok(())
    /// # }
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
/// use az_regex::{CachedRegex, named_captures_to_map};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let re = CachedRegex::new(r"(?<host>[^:]+):(?<port>\d+)")?;
/// let caps = re.captures("localhost:8080").ok_or("missing captures")?;
/// let map = named_captures_to_map(re.regex(), &caps);
/// assert_eq!(map["host"], "localhost");
/// assert_eq!(map["port"], "8080");
/// # Ok(())
/// # }
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
/// use az_regex::extract_all;
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
/// use az_regex::is_valid_pattern;
///
/// assert!(is_valid_pattern(r"\d+"));
/// assert!(!is_valid_pattern("[invalid"));
/// ```
#[must_use]
pub fn is_valid_pattern(pattern: &str) -> bool {
    Regex::new(pattern).is_ok()
}
