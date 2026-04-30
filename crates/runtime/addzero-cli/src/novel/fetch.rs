use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::web::robots::RobotsPolicy;
use crate::web::{
    build_client, decode_html, ensure_parent_dir, first_non_empty_text, manifest_path, output_stem,
    parse_selector,
};

use super::cleanup::clean_body;
use super::preset::{NovelPreset, ResolvedSelectors, resolve_selectors};
use super::toc::{ParsedToc, TocParser};

#[derive(Debug, Clone)]
pub struct NovelFetchConfig {
    pub toc_url: Url,
    pub output: PathBuf,
    pub preset: NovelPreset,
    pub book_title_selector: Option<String>,
    pub chapter_list_selector: Option<String>,
    pub chapter_link_selector: Option<String>,
    pub chapter_title_selector: Option<String>,
    pub content_selectors: Vec<String>,
    pub delay: Duration,
    pub user_agent: String,
    pub max_chapters: Option<usize>,
    pub allow_offsite: bool,
    pub ignore_robots: bool,
}

#[derive(Debug, Serialize)]
struct NovelManifest {
    toc_url: String,
    book_title: String,
    preset: String,
    chapter_count: usize,
    chapters: Vec<ChapterManifest>,
}

#[derive(Debug, Serialize)]
struct ChapterManifest {
    index: usize,
    title: String,
    url: String,
    char_count: usize,
}

#[derive(Debug)]
struct DownloadedChapter {
    index: usize,
    title: String,
    url: Url,
    body: String,
}

#[derive(Debug)]
struct ChapterExtractor {
    chapter_title_selectors: Vec<Selector>,
    content_selectors: Vec<Selector>,
    paragraph_selector: Selector,
}

impl NovelFetchConfig {
    fn validate(&self) -> Result<()> {
        if matches!(self.max_chapters, Some(0)) {
            anyhow::bail!("--max-chapters must be greater than 0");
        }
        Ok(())
    }
}

impl ChapterExtractor {
    fn from_resolved(selectors: &ResolvedSelectors) -> Result<Self> {
        let chapter_title_selectors = selectors
            .chapter_title_selectors
            .iter()
            .map(|value| parse_selector(value, "chapter title selector"))
            .collect::<Result<Vec<_>>>()?;
        let content_selectors = selectors
            .content_selectors
            .iter()
            .map(|value| parse_selector(value, "content selector"))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            chapter_title_selectors,
            content_selectors,
            paragraph_selector: parse_selector("p", "paragraph selector")?,
        })
    }

    fn extract(
        &self,
        index: usize,
        url: &Url,
        html: &str,
        fallback_title: Option<&str>,
        preset: NovelPreset,
    ) -> Result<DownloadedChapter> {
        let document = Html::parse_document(html);
        let title = self.extract_title(&document, fallback_title, index);
        let body = self
            .extract_body(&document, preset)?
            .with_context(|| format!("no content matched for {}", url.as_str()))?;

        Ok(DownloadedChapter {
            index,
            title,
            url: url.clone(),
            body,
        })
    }

    fn extract_title(&self, document: &Html, fallback_title: Option<&str>, index: usize) -> String {
        for selector in &self.chapter_title_selectors {
            if let Some(title) = first_non_empty_text(document, selector) {
                return title;
            }
        }

        fallback_title
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Chapter {index}"))
    }

    fn extract_body(&self, document: &Html, preset: NovelPreset) -> Result<Option<String>> {
        let mut seen = HashSet::new();

        for selector in &self.content_selectors {
            let mut blocks = Vec::new();

            for element in document.select(selector) {
                for block in self.extract_blocks_from_element(&element, preset) {
                    if seen.insert(block.clone()) {
                        blocks.push(block);
                    }
                }
            }

            if !blocks.is_empty() {
                return Ok(Some(blocks.join("\n\n")));
            }
        }

        Ok(None)
    }

    fn extract_blocks_from_element(
        &self,
        element: &ElementRef<'_>,
        preset: NovelPreset,
    ) -> Vec<String> {
        let paragraph_blocks = element
            .select(&self.paragraph_selector)
            .filter_map(|paragraph| {
                let cleaned = clean_body(&raw_text(&paragraph), preset);
                if cleaned.is_empty() {
                    None
                } else {
                    Some(cleaned)
                }
            })
            .collect::<Vec<_>>();

        if !paragraph_blocks.is_empty() {
            return paragraph_blocks;
        }

        let cleaned = clean_body(&raw_text(element), preset);
        if cleaned.is_empty() {
            Vec::new()
        } else {
            vec![cleaned]
        }
    }
}

pub fn run_fetch(config: NovelFetchConfig) -> Result<()> {
    config.validate()?;

    let resolved = resolve_selectors(&config)?;
    let toc_parser = TocParser::from_resolved(&resolved)?;
    let chapter_extractor = ChapterExtractor::from_resolved(&resolved)?;
    let client = build_client(&config.user_agent)?;

    let robots = if config.ignore_robots {
        None
    } else {
        Some(RobotsPolicy::load(
            &client,
            &config.toc_url,
            &config.user_agent,
        )?)
    };

    if let Some(policy) = robots.as_ref() {
        policy.ensure_allowed(&config.toc_url)?;
    }

    ensure_parent_dir(&config.output)?;

    let toc_response = client
        .get(config.toc_url.clone())
        .send()
        .with_context(|| format!("request TOC {}", config.toc_url.as_str()))?
        .error_for_status()
        .with_context(|| format!("non-success response from {}", config.toc_url.as_str()))?;
    let toc_html = decode_html(toc_response)?;
    let mut toc = toc_parser.parse(
        &toc_html,
        &config.toc_url,
        &output_stem(&config.output, "novel"),
        config.allow_offsite,
    )?;

    if let Some(max) = config.max_chapters {
        toc.chapters.truncate(max);
    }

    if toc.chapters.is_empty() {
        anyhow::bail!("no chapters discovered after applying filters");
    }

    let chapters = download_chapters(&client, robots.as_ref(), &chapter_extractor, &toc, &config)?;

    let output = render_output(&chapters);
    fs::write(&config.output, output)
        .with_context(|| format!("write output file: {}", config.output.display()))?;

    let manifest = NovelManifest {
        toc_url: config.toc_url.as_str().to_owned(),
        book_title: toc.book_title.clone(),
        preset: config.preset.as_str().to_owned(),
        chapter_count: chapters.len(),
        chapters: chapters
            .iter()
            .map(|chapter| ChapterManifest {
                index: chapter.index,
                title: chapter.title.clone(),
                url: chapter.url.as_str().to_owned(),
                char_count: chapter.body.chars().count(),
            })
            .collect(),
    };

    let manifest_path = manifest_path(&config.output, "novel");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write manifest: {}", manifest_path.display()))?;

    println!(
        "saved {} chapter(s) from {} to {}",
        chapters.len(),
        toc.book_title,
        config.output.display()
    );
    Ok(())
}

fn download_chapters(
    client: &reqwest::blocking::Client,
    robots: Option<&RobotsPolicy>,
    chapter_extractor: &ChapterExtractor,
    toc: &ParsedToc,
    config: &NovelFetchConfig,
) -> Result<Vec<DownloadedChapter>> {
    let mut chapters = Vec::with_capacity(toc.chapters.len());

    for (zero_index, entry) in toc.chapters.iter().enumerate() {
        let index = zero_index + 1;

        if let Some(policy) = robots {
            policy.ensure_allowed(&entry.url)?;
        }

        if index > 1 && !config.delay.is_zero() {
            sleep(config.delay);
        }

        let response = client
            .get(entry.url.clone())
            .send()
            .with_context(|| format!("request chapter {}", entry.url.as_str()))?
            .error_for_status()
            .with_context(|| format!("non-success response from {}", entry.url.as_str()))?;
        let html = decode_html(response)?;
        let chapter = chapter_extractor.extract(
            index,
            &entry.url,
            &html,
            entry.title.as_deref(),
            config.preset,
        )?;

        chapters.push(chapter);
    }

    Ok(chapters)
}

fn raw_text(element: &ElementRef<'_>) -> String {
    element.text().collect::<Vec<_>>().join("\n")
}

fn render_output(chapters: &[DownloadedChapter]) -> String {
    let mut output = String::new();
    for (idx, chapter) in chapters.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        output.push_str(&chapter.title);
        output.push_str("\n\n");
        output.push_str(&chapter.body);
        output.push_str("\n\n");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use reqwest::Url;

    use super::*;

    #[test]
    fn chapter_extractor_prefers_toc_title_when_page_title_missing() {
        let config = sample_config(NovelPreset::Xbqg);
        let resolved = resolve_selectors(&config).expect("selectors resolve");
        let extractor = ChapterExtractor::from_resolved(&resolved).expect("extractor builds");
        let html = "<html><body><div id=\"content\">第一段。\
\u{00a0}\u{00a0}\u{00a0}\u{00a0}第二段。</div></body></html>";

        let chapter = extractor
            .extract(
                3,
                &Url::parse("https://example.com/chapter-3").expect("url"),
                html,
                Some("第三章 预言"),
                NovelPreset::Xbqg,
            )
            .expect("chapter extracts");

        assert_eq!(chapter.title, "第三章 预言");
        assert_eq!(chapter.body, "第一段。\n\n第二段。");
    }

    #[test]
    fn chapter_extractor_prefers_page_title_and_then_chapter_number() {
        let config = sample_config(NovelPreset::Xbqg);
        let resolved = resolve_selectors(&config).expect("selectors resolve");
        let extractor = ChapterExtractor::from_resolved(&resolved).expect("extractor builds");

        let titled_html =
            "<html><body><h1>第七章 远行</h1><div id=\"content\">正文。</div></body></html>";
        let titled = extractor
            .extract(
                7,
                &Url::parse("https://example.com/chapter-7").expect("url"),
                titled_html,
                Some("TOC 标题"),
                NovelPreset::Xbqg,
            )
            .expect("chapter extracts");
        assert_eq!(titled.title, "第七章 远行");

        let untitled_html = "<html><body><div id=\"content\">正文。</div></body></html>";
        let untitled = extractor
            .extract(
                9,
                &Url::parse("https://example.com/chapter-9").expect("url"),
                untitled_html,
                None,
                NovelPreset::Xbqg,
            )
            .expect("chapter extracts");
        assert_eq!(untitled.title, "Chapter 9");
    }

    #[test]
    fn manifest_like_data_uses_clean_char_counts() {
        let chapters = [DownloadedChapter {
            index: 1,
            title: "第一章".to_owned(),
            url: Url::parse("https://example.com/1").expect("url"),
            body: "第一段。\n\n第二段。".to_owned(),
        }];

        let manifest = NovelManifest {
            toc_url: "https://example.com/book".to_owned(),
            book_title: "示例".to_owned(),
            preset: "xbqg".to_owned(),
            chapter_count: chapters.len(),
            chapters: chapters
                .iter()
                .map(|chapter| ChapterManifest {
                    index: chapter.index,
                    title: chapter.title.clone(),
                    url: chapter.url.as_str().to_owned(),
                    char_count: chapter.body.chars().count(),
                })
                .collect(),
        };

        assert_eq!(manifest.chapter_count, 1);
        assert_eq!(
            manifest.chapters[0].char_count,
            "第一段。\n\n第二段。".chars().count()
        );
    }

    #[test]
    fn config_rejects_zero_max_chapters() {
        let mut config = sample_config(NovelPreset::Xbqg);
        config.max_chapters = Some(0);

        assert!(config.validate().is_err());
    }

    fn sample_config(preset: NovelPreset) -> NovelFetchConfig {
        NovelFetchConfig {
            toc_url: Url::parse("https://example.com/book").expect("url"),
            output: PathBuf::from("book.txt"),
            preset,
            book_title_selector: None,
            chapter_list_selector: None,
            chapter_link_selector: Some("a[href]".to_owned()),
            chapter_title_selector: None,
            content_selectors: vec!["#content".to_owned()],
            delay: Duration::from_millis(0),
            user_agent: "test-agent".to_owned(),
            max_chapters: None,
            allow_offsite: false,
            ignore_robots: false,
        }
    }
}
