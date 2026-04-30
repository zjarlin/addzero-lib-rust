use std::sync::OnceLock;

use regex::Regex;

use super::preset::NovelPreset;

pub fn clean_body(raw: &str, preset: NovelPreset) -> String {
    let mut normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    normalized = paragraph_breaks()
        .replace_all(&normalized, "\n")
        .into_owned();
    normalized = normalized.replace("&nbsp;", " ");

    let mut blocks = Vec::new();
    for line in normalized.lines() {
        let collapsed = line
            .replace(['\u{feff}', '\u{00a0}', '\u{3000}'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let collapsed = collapsed.trim();
        if collapsed.is_empty() || is_noise_line(collapsed, preset) {
            continue;
        }
        blocks.push(collapsed.to_owned());
    }

    blocks.join("\n\n")
}

fn paragraph_breaks() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?:\u{00a0}|\u{3000}){4,}").expect("regex compiles"))
}

fn is_noise_line(line: &str, preset: NovelPreset) -> bool {
    let lower = line.to_ascii_lowercase();
    let common_markers = [
        "最新网址",
        "请收藏",
        "手机用户请浏览",
        "天才一秒记住",
        "一秒记住",
        "chaptererror()",
        "readx();",
    ];
    if common_markers.iter().any(|marker| line.contains(marker)) {
        return true;
    }

    match preset {
        NovelPreset::Biqukan => lower.contains("biqukan.com") || line.contains("笔趣看"),
        NovelPreset::Xbqg => lower.contains("xsbiquge.com") || line.contains("笔趣阁"),
        NovelPreset::Custom => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_removes_nbsp_runs_and_watermark_lines() {
        let raw = "\
\u{00a0}\u{00a0}\u{00a0}\u{00a0}第一段。\n\
最新网址：xsbiquge.com\n\
\u{3000}\u{3000}\u{3000}\u{3000}第二段。";

        let cleaned = clean_body(raw, NovelPreset::Xbqg);

        assert_eq!(cleaned, "第一段。\n\n第二段。");
    }
}
