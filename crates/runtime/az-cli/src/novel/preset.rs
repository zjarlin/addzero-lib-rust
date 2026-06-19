use anyhow::Result;

use super::NovelFetchConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NovelPreset {
    Biqukan,
    Xbqg,
    Custom,
}

impl NovelPreset {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSelectors {
    pub book_title_selectors: Vec<String>,
    pub chapter_list_selectors: Vec<String>,
    pub chapter_link_selectors: Vec<String>,
    pub chapter_title_selectors: Vec<String>,
    pub content_selectors: Vec<String>,
}

pub fn resolve_selectors(config: &NovelFetchConfig) -> Result<ResolvedSelectors> {
    if matches!(config.preset, NovelPreset::Custom) {
        if config.chapter_link_selector.is_none() {
            anyhow::bail!("--preset custom requires --chapter-link-selector");
        }
        if config.content_selectors.is_empty() {
            anyhow::bail!("--preset custom requires at least one --content-selector");
        }
    }

    let defaults = preset_defaults(config.preset);
    let resolved = ResolvedSelectors {
        book_title_selectors: override_single(
            defaults.book_title_selectors,
            config.book_title_selector.as_deref(),
        ),
        chapter_list_selectors: override_single(
            defaults.chapter_list_selectors,
            config.chapter_list_selector.as_deref(),
        ),
        chapter_link_selectors: override_single(
            defaults.chapter_link_selectors,
            config.chapter_link_selector.as_deref(),
        ),
        chapter_title_selectors: override_single(
            defaults.chapter_title_selectors,
            config.chapter_title_selector.as_deref(),
        ),
        content_selectors: override_multi(defaults.content_selectors, &config.content_selectors),
    };

    if resolved.chapter_link_selectors.is_empty() {
        anyhow::bail!("no chapter link selector configured");
    }
    if resolved.content_selectors.is_empty() {
        anyhow::bail!("no content selector configured");
    }

    Ok(resolved)
}

struct PresetDefaults {
    book_title_selectors: &'static [&'static str],
    chapter_list_selectors: &'static [&'static str],
    chapter_link_selectors: &'static [&'static str],
    chapter_title_selectors: &'static [&'static str],
    content_selectors: &'static [&'static str],
}

fn preset_defaults(preset: NovelPreset) -> PresetDefaults {
    match preset {
        NovelPreset::Biqukan => PresetDefaults {
            book_title_selectors: &["div.book div.info h2", ".book .info h2", "div.info h2"],
            chapter_list_selectors: &["div.listmain"],
            chapter_link_selectors: &["dl dd a[href]", "a[href]"],
            chapter_title_selectors: &["h1"],
            content_selectors: &["#content", ".showtxt", "div.showtxt"],
        },
        NovelPreset::Xbqg => PresetDefaults {
            book_title_selectors: &["#info h1", "div#info h1"],
            chapter_list_selectors: &["#list", "div#list"],
            chapter_link_selectors: &["a[href]"],
            chapter_title_selectors: &["h1"],
            content_selectors: &["#content", "div#content", ".content"],
        },
        NovelPreset::Custom => PresetDefaults {
            book_title_selectors: &[],
            chapter_list_selectors: &[],
            chapter_link_selectors: &[],
            chapter_title_selectors: &["h1"],
            content_selectors: &[],
        },
    }
}

fn override_single(defaults: &[&str], override_value: Option<&str>) -> Vec<String> {
    match override_value {
        Some(value) => vec![value.to_owned()],
        None => defaults.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn override_multi(defaults: &[&str], override_values: &[String]) -> Vec<String> {
    if override_values.is_empty() {
        defaults.iter().map(|value| (*value).to_owned()).collect()
    } else {
        override_values.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use reqwest::Url;

    use super::*;

    #[test]
    fn user_overrides_replace_preset_category_defaults() {
        let config = sample_config(NovelPreset::Biqukan);
        let resolved = resolve_selectors(&config).expect("selectors resolve");

        assert_eq!(resolved.chapter_list_selectors, vec!["section.toc"]);
        assert_eq!(resolved.chapter_title_selectors, vec![".chapter-title"]);
        assert_eq!(resolved.content_selectors, vec!["article", "#reader"]);
        assert_eq!(
            resolved.chapter_link_selectors,
            vec!["dl dd a[href]", "a[href]"]
        );
    }

    #[test]
    fn custom_preset_requires_link_and_content_selectors() {
        let mut config = sample_config(NovelPreset::Custom);
        config.chapter_link_selector = None;

        assert!(resolve_selectors(&config).is_err());

        let mut config = sample_config(NovelPreset::Custom);
        config.content_selectors.clear();

        assert!(resolve_selectors(&config).is_err());
    }

    #[test]
    fn custom_preset_accepts_explicit_positive_configuration() {
        let mut config = sample_config(NovelPreset::Custom);
        config.book_title_selector = Some(".book-title".to_owned());

        let resolved = resolve_selectors(&config).expect("selectors resolve");

        assert_eq!(resolved.book_title_selectors, vec![".book-title"]);
        assert_eq!(resolved.chapter_link_selectors, vec!["a[href]"]);
        assert_eq!(resolved.content_selectors, vec!["article", "#reader"]);
        assert_eq!(resolved.chapter_title_selectors, vec![".chapter-title"]);
    }

    #[test]
    fn preset_codes_round_trip_through_serde() {
        let json = serde_json::to_string(&NovelPreset::Xbqg).expect("serialize preset");
        assert_eq!(json, "\"xbqg\"");
        let preset: NovelPreset = serde_json::from_str("\"custom\"").expect("deserialize preset");
        assert_eq!(preset, NovelPreset::Custom);
        assert_eq!(NovelPreset::Biqukan.as_str(), "biqukan");
    }

    fn sample_config(preset: NovelPreset) -> NovelFetchConfig {
        NovelFetchConfig {
            toc_url: Url::parse("https://example.com/book").expect("url"),
            output: PathBuf::from("book.txt"),
            preset,
            book_title_selector: None,
            chapter_list_selector: Some("section.toc".to_owned()),
            chapter_link_selector: matches!(preset, NovelPreset::Custom)
                .then(|| "a[href]".to_owned()),
            chapter_title_selector: Some(".chapter-title".to_owned()),
            content_selectors: vec!["article".to_owned(), "#reader".to_owned()],
            delay: Duration::from_millis(0),
            user_agent: "test-agent".to_owned(),
            max_chapters: None,
            allow_offsite: false,
            ignore_robots: false,
        }
    }
}
