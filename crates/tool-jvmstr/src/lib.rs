use deunicode::deunicode;
use regex::Regex;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub fn default_table_english_name(
    table_english_name: impl AsRef<str>,
    table_chinese_name: Option<&str>,
) -> String {
    let table_english_name = table_english_name.as_ref();
    let seed = if table_english_name.trim().is_empty() {
        table_chinese_name.unwrap_or_default()
    } else {
        table_english_name
    };

    let without_parenthetical = parenthetical_regex().replace_all(seed, "");
    let transliterated = if table_english_name.trim().is_empty() {
        deunicode(without_parenthetical.trim())
    } else {
        without_parenthetical.to_string()
    };

    let sanitized: String = transliterated
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();

    underscore_regex()
        .replace_all(sanitized.trim_matches('_'), "_")
        .to_string()
}

pub fn parent_path_and_mkdir(
    path: impl AsRef<Path>,
    child_path: impl AsRef<Path>,
) -> io::Result<PathBuf> {
    let path = path.as_ref();
    let Some(parent) = path.parent() else {
        let target = PathBuf::from(child_path.as_ref());
        fs::create_dir_all(&target)?;
        return Ok(target);
    };

    let target = parent.join(child_path.as_ref());
    fs::create_dir_all(&target)?;
    Ok(target)
}

pub trait ParentPathExt {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>;
}

impl ParentPathExt for Path {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        parent_path_and_mkdir(self, child_path)
    }
}

impl ParentPathExt for str {
    fn parent_path_and_mkdir<P>(&self, child_path: P) -> io::Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        parent_path_and_mkdir(self, child_path)
    }
}

pub fn extract_markdown_block_content(markdown: Option<&str>) -> String {
    let Some(markdown) = markdown else {
        return String::new();
    };
    if markdown.is_empty() {
        return String::new();
    }

    if markdown.contains("```") || markdown.contains("json") {
        return fenced_block_regex()
            .captures(markdown)
            .and_then(|captures| captures.get(1))
            .map(|matched| matched.as_str().trim().to_owned())
            .unwrap_or_default();
    }

    markdown.to_owned()
}

pub fn extract_code_block_content(code: impl AsRef<str>) -> String {
    double_tick_regex()
        .captures(code.as_ref())
        .and_then(|captures| captures.get(1))
        .map(|matched| matched.as_str().trim().to_owned())
        .unwrap_or_default()
}

fn parenthetical_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\((.*?)\)").expect("regex must compile"))
}

fn underscore_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"_{2,}").expect("regex must compile"))
}

fn fenced_block_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)```[\w-]*\s*(.*?)\s*```").expect("regex must compile"))
}

fn double_tick_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?s)``\w*\s*(.*?)\s*``").expect("regex must compile"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
                .all(|character| { character.is_ascii_alphanumeric() || character == '_' })
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
    fn extract_markdown_block_content_returns_first_fenced_block() {
        let markdown = "before\n```json\n{\"name\":\"addzero\"}\n```\nafter";

        let extracted = extract_markdown_block_content(Some(markdown));

        assert_eq!(extracted, "{\"name\":\"addzero\"}");
    }

    #[test]
    fn extract_markdown_block_content_returns_raw_text_without_fence() {
        let markdown = "plain text";

        let extracted = extract_markdown_block_content(Some(markdown));

        assert_eq!(extracted, "plain text");
    }

    #[test]
    fn extract_code_block_content_reads_double_tick_fence() {
        let code = "``sql\nselect * from users\n``";

        let extracted = extract_code_block_content(code);

        assert_eq!(extracted, "select * from users");
    }
}
