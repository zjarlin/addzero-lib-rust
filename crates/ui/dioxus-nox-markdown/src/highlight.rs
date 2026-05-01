//! Syntax highlighting for fenced code blocks.
//!
//! When the `syntax-highlighting` feature is enabled, uses `syntect` to produce
//! CSS-class-annotated `<span>` elements (e.g. `<span class="hl-keyword">`).
//! Consumers supply their own CSS — this crate ships zero visual styles.
//!
//! When the feature is disabled, all public functions degrade gracefully:
//! `highlight_code` returns HTML-escaped plain text, `generate_theme_css`
//! returns `None`, and `supported_languages` returns an empty `Vec`.

/// Result of highlighting a code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightResult {
    /// Highlighted HTML (or HTML-escaped plain text when feature is off / language unknown).
    pub html: String,
    /// Whether syntect recognized the language token.
    pub language_matched: bool,
}

// ── Feature-gated implementation ────────────────────────────────────

#[cfg(feature = "syntax-highlighting")]
mod inner {
    use super::HighlightResult;
    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex};
    use syntect::highlighting::ThemeSet;
    use syntect::html::ClassedHTMLGenerator;
    use syntect::parsing::SyntaxSet;

    static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

    /// Cache of leaked prefix strings for `ClassStyle::SpacedPrefixed`.
    ///
    /// `ClassedHTMLGenerator` ties the `ClassStyle` lifetime to the `SyntaxSet`
    /// lifetime (`'static` via `LazyLock`). We leak each unique prefix once to
    /// produce `&'static str`. In practice only 1-2 prefixes are used per app.
    static PREFIX_CACHE: LazyLock<Mutex<HashMap<String, &'static str>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    fn static_prefix(prefix: &str) -> &'static str {
        let mut cache = PREFIX_CACHE.lock().unwrap();
        if let Some(&s) = cache.get(prefix) {
            return s;
        }
        // Intentional bounded leak: each unique prefix is leaked exactly once and
        // cached in PREFIX_CACHE. In practice only 1-2 prefixes exist per app,
        // so total leaked memory is negligible (~10-50 bytes).
        let leaked: &'static str = Box::leak(prefix.to_string().into_boxed_str());
        cache.insert(prefix.to_string(), leaked);
        leaked
    }

    /// Highlight `code` with the syntax definition for `lang`.
    ///
    /// Returns `HighlightResult` with `language_matched = true` when syntect
    /// recognizes the language, or HTML-escaped plain text otherwise.
    ///
    /// `class_prefix` is prepended to every CSS class name (default `"hl-"`).
    pub fn highlight_code(code: &str, lang: &str, class_prefix: &str) -> HighlightResult {
        let syntax = if lang.is_empty() {
            None
        } else {
            SYNTAX_SET
                .find_syntax_by_token(lang)
                .or_else(|| SYNTAX_SET.find_syntax_by_name(lang))
        };

        let Some(syntax) = syntax else {
            return HighlightResult {
                html: html_escape(code),
                language_matched: false,
            };
        };

        let class_style = if class_prefix.is_empty() {
            syntect::html::ClassStyle::Spaced
        } else {
            syntect::html::ClassStyle::SpacedPrefixed {
                prefix: static_prefix(class_prefix),
            }
        };

        let mut generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAX_SET, class_style);

        for line in syntect::util::LinesWithEndings::from(code) {
            if generator
                .parse_html_for_line_which_includes_newline(line)
                .is_err()
            {
                return HighlightResult {
                    html: html_escape(code),
                    language_matched: false,
                };
            }
        }

        HighlightResult {
            html: generator.finalize(),
            language_matched: true,
        }
    }

    /// Generate a CSS stylesheet for a named syntect theme (e.g. `"base16-ocean.dark"`).
    ///
    /// Each rule uses `class_prefix` (e.g. `.hl-keyword { color: ... }`).
    /// Returns `None` if the theme name is not found.
    pub fn generate_theme_css(theme_name: &str, class_prefix: &str) -> Option<String> {
        let theme = THEME_SET.themes.get(theme_name)?;
        let class_style = if class_prefix.is_empty() {
            syntect::html::ClassStyle::Spaced
        } else {
            syntect::html::ClassStyle::SpacedPrefixed {
                prefix: static_prefix(class_prefix),
            }
        };
        syntect::html::css_for_theme_with_class_style(theme, class_style).ok()
    }

    /// List all language tokens recognized by the default `SyntaxSet`.
    pub fn supported_languages() -> Vec<&'static str> {
        SYNTAX_SET
            .syntaxes()
            .iter()
            .flat_map(|s| s.file_extensions.iter().map(|e| e.as_str()))
            .collect()
    }

    /// Minimal HTML escaping for untrusted code text.
    fn html_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                _ => out.push(ch),
            }
        }
        out
    }
}

// ── Fallback (feature disabled) ─────────────────────────────────────

#[cfg(not(feature = "syntax-highlighting"))]
mod inner {
    use super::HighlightResult;

    /// Returns HTML-escaped plain text (no highlighting without the feature).
    pub fn highlight_code(code: &str, _lang: &str, _class_prefix: &str) -> HighlightResult {
        HighlightResult {
            html: html_escape(code),
            language_matched: false,
        }
    }

    /// Always returns `None` without the `syntax-highlighting` feature.
    pub fn generate_theme_css(_theme_name: &str, _class_prefix: &str) -> Option<String> {
        None
    }

    /// Returns an empty list without the `syntax-highlighting` feature.
    pub fn supported_languages() -> Vec<&'static str> {
        Vec::new()
    }

    fn html_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                _ => out.push(ch),
            }
        }
        out
    }
}

// ── Public re-exports ───────────────────────────────────────────────

pub use inner::{generate_theme_css, highlight_code, supported_languages};

/// Wraps highlighted (or plain) HTML code with line number gutter spans.
///
/// Each line is wrapped in a `<span class="code-line" data-line-number="N">` container.
/// The line number itself is a non-selectable `<span>` with `data-md-line-gutter`,
/// `aria-hidden="true"`, and `user-select:none` (FUNCTIONAL — copy-paste behavior).
///
/// Works identically whether syntax-highlighting is on or off (operates on the
/// HTML string output from `highlight_code`).
pub fn wrap_with_line_numbers(html: &str) -> String {
    let lines: Vec<&str> = html.split('\n').collect();
    // Trim trailing empty line (syntect often ends with a trailing newline)
    let line_count = if lines.last() == Some(&"") && lines.len() > 1 {
        lines.len() - 1
    } else {
        lines.len()
    };

    let mut out = String::with_capacity(html.len() + line_count * 100);
    for (i, line) in lines.iter().take(line_count).enumerate() {
        let num = i + 1;
        out.push_str(&format!(
            "<span class=\"code-line\" data-line-number=\"{num}\"><span data-md-line-gutter aria-hidden=\"true\" style=\"user-select:none\">{num}</span>{line}</span>\n",
        ));
    }
    // Remove trailing newline from the last push
    if out.ends_with('\n') {
        out.pop();
    }
    out
}
