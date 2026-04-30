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
    build_client, decode_html, ensure_parent_dir, first_non_empty_text, href_from_element,
    manifest_path, normalize_inline_whitespace, normalized_text, parse_selector, same_origin,
};

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub start_url: Url,
    pub output: PathBuf,
    pub content_selectors: Vec<String>,
    pub title_selector: Option<String>,
    pub next_selectors: Vec<String>,
    pub max_pages: usize,
    pub delay: Duration,
    pub user_agent: String,
    pub allow_offsite: bool,
    pub ignore_robots: bool,
}

#[derive(Debug, Serialize)]
struct DownloadManifest {
    start_url: String,
    page_count: usize,
    output: String,
    pages: Vec<PageManifest>,
}

#[derive(Debug, Serialize)]
struct PageManifest {
    index: usize,
    title: String,
    url: String,
    char_count: usize,
}

#[derive(Debug)]
struct DownloadedPage {
    index: usize,
    title: String,
    url: Url,
    body: String,
    next_url: Option<Url>,
}

#[derive(Debug)]
struct Extractor {
    content_selectors: Vec<Selector>,
    title_selector: Option<Selector>,
    next_selectors: Vec<Selector>,
}

impl Extractor {
    fn from_config(config: &DownloadConfig) -> Result<Self> {
        let content_selectors = config
            .content_selectors
            .iter()
            .map(|value| parse_selector(value, "content selector"))
            .collect::<Result<Vec<_>>>()?;

        let title_selector = config
            .title_selector
            .as_deref()
            .map(|value| parse_selector(value, "title selector"))
            .transpose()?;

        let next_selectors = config
            .next_selectors
            .iter()
            .map(|value| parse_selector(value, "next selector"))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            content_selectors,
            title_selector,
            next_selectors,
        })
    }

    fn extract_page(&self, index: usize, url: &Url, html: &str) -> Result<DownloadedPage> {
        let document = Html::parse_document(html);
        let title = self
            .extract_title(&document)?
            .unwrap_or_else(|| format!("Page {index}"));
        let body = self
            .extract_body(&document)?
            .with_context(|| format!("no content matched for {}", url.as_str()))?;
        let next_url = self.extract_next_url(&document, url)?;

        Ok(DownloadedPage {
            index,
            title,
            url: url.clone(),
            body,
            next_url,
        })
    }

    fn extract_title(&self, document: &Html) -> Result<Option<String>> {
        if let Some(selector) = &self.title_selector {
            if let Some(title) = first_non_empty_text(document, selector) {
                return Ok(Some(title));
            }
        }

        let h1_selector = parse_selector("h1", "fallback h1 selector")?;
        if let Some(title) = first_non_empty_text(document, &h1_selector) {
            return Ok(Some(title));
        }

        let title_selector = parse_selector("title", "fallback title selector")?;
        Ok(first_non_empty_text(document, &title_selector))
    }

    fn extract_body(&self, document: &Html) -> Result<Option<String>> {
        for selector in &self.content_selectors {
            let blocks = extract_blocks(document, selector)?;
            if !blocks.is_empty() {
                return Ok(Some(blocks.join("\n\n")));
            }
        }

        Ok(None)
    }

    fn extract_next_url(&self, document: &Html, base_url: &Url) -> Result<Option<Url>> {
        for selector in &self.next_selectors {
            for element in document.select(selector) {
                if let Some(url) = href_from_element(base_url, &element) {
                    return Ok(Some(url));
                }
            }
        }

        infer_next_url(document, base_url)
    }
}

pub fn run_download(config: DownloadConfig) -> Result<()> {
    let extractor = Extractor::from_config(&config)?;
    let client = build_client(&config.user_agent)?;

    let robots = if config.ignore_robots {
        None
    } else {
        Some(RobotsPolicy::load(
            &client,
            &config.start_url,
            &config.user_agent,
        )?)
    };

    ensure_parent_dir(&config.output)?;

    let mut current_url = config.start_url.clone();
    let mut visited = HashSet::new();
    let mut pages = Vec::new();

    for index in 1..=config.max_pages {
        if !visited.insert(current_url.as_str().to_owned()) {
            break;
        }

        if let Some(policy) = robots.as_ref() {
            policy.ensure_allowed(&current_url)?;
        }

        if index > 1 && !config.delay.is_zero() {
            sleep(config.delay);
        }

        let response = client
            .get(current_url.clone())
            .send()
            .with_context(|| format!("request {}", current_url.as_str()))?
            .error_for_status()
            .with_context(|| format!("non-success response from {}", current_url.as_str()))?;

        let html = decode_html(response)?;
        let mut page = extractor.extract_page(index, &current_url, &html)?;

        if let Some(next_url) = page.next_url.as_ref() {
            if !config.allow_offsite && !same_origin(&config.start_url, next_url) {
                page.next_url = None;
            }
        }

        current_url = match page.next_url.clone() {
            Some(next_url) => next_url,
            None => {
                pages.push(page);
                break;
            }
        };

        pages.push(page);
    }

    if pages.is_empty() {
        anyhow::bail!("no pages downloaded");
    }

    let body = render_output(&pages);
    fs::write(&config.output, body)
        .with_context(|| format!("write output file: {}", config.output.display()))?;

    let manifest = DownloadManifest {
        start_url: config.start_url.as_str().to_owned(),
        page_count: pages.len(),
        output: config.output.display().to_string(),
        pages: pages
            .iter()
            .map(|page| PageManifest {
                index: page.index,
                title: page.title.clone(),
                url: page.url.as_str().to_owned(),
                char_count: page.body.chars().count(),
            })
            .collect(),
    };

    let manifest_path = manifest_path(&config.output, "web-text");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write manifest: {}", manifest_path.display()))?;

    println!(
        "saved {} page(s) to {}",
        pages.len(),
        config.output.display()
    );
    Ok(())
}

fn extract_blocks(document: &Html, selector: &Selector) -> Result<Vec<String>> {
    let paragraph_selector = parse_selector("p", "paragraph selector")?;
    let mut blocks = Vec::new();
    let mut seen = HashSet::new();

    for element in document.select(selector) {
        let paragraph_blocks = element
            .select(&paragraph_selector)
            .filter_map(|paragraph| normalized_text(&paragraph))
            .collect::<Vec<_>>();

        if !paragraph_blocks.is_empty() {
            for block in paragraph_blocks {
                if seen.insert(block.clone()) {
                    blocks.push(block);
                }
            }
            continue;
        }

        if let Some(block) = normalized_text(&element) {
            if seen.insert(block.clone()) {
                blocks.push(block);
            }
        }
    }

    Ok(blocks)
}

fn infer_next_url(document: &Html, base_url: &Url) -> Result<Option<Url>> {
    let link_selector = parse_selector("a[href]", "anchor selector")?;
    for element in document.select(&link_selector) {
        if looks_like_next_link(&element) {
            if let Some(url) = href_from_element(base_url, &element) {
                return Ok(Some(url));
            }
        }
    }
    Ok(None)
}

fn looks_like_next_link(element: &ElementRef<'_>) -> bool {
    let rel = element
        .value()
        .attr("rel")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if rel.contains("next") {
        return true;
    }

    let identity = [
        element.value().attr("id").unwrap_or_default(),
        element.value().attr("class").unwrap_or_default(),
    ]
    .join(" ")
    .to_ascii_lowercase();
    if identity.contains("next") {
        return true;
    }

    let text = element
        .text()
        .map(normalize_inline_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();

    matches!(
        text.as_str(),
        "next"
            | "next page"
            | "next chapter"
            | ">"
            | ">>"
            | "›"
            | "»"
            | "下一页"
            | "下一章"
            | "下页"
            | "下章"
            | "下一话"
    ) || text.contains("next chapter")
}

fn render_output(pages: &[DownloadedPage]) -> String {
    let mut output = String::new();
    for (idx, page) in pages.iter().enumerate() {
        if idx > 0 {
            output.push_str("\n\n---\n\n");
        }

        output.push_str("# ");
        output.push_str(&page.title);
        output.push_str("\n\n");
        output.push_str("Source: ");
        output.push_str(page.url.as_str());
        output.push_str("\n\n");
        output.push_str(&page.body);
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extractor_should_capture_title_body_and_next_link() {
        let config = DownloadConfig {
            start_url: Url::parse("https://example.com/book/1").expect("valid url"),
            output: PathBuf::from("novel.txt"),
            content_selectors: vec!["article.chapter".to_owned()],
            title_selector: Some("h1.chapter-title".to_owned()),
            next_selectors: vec!["a.next".to_owned()],
            max_pages: 10,
            delay: Duration::from_millis(0),
            user_agent: "test-agent".to_owned(),
            allow_offsite: false,
            ignore_robots: false,
        };
        let extractor = Extractor::from_config(&config).expect("extractor builds");
        let html = r#"
            <html>
                <body>
                    <article class="chapter">
                        <h1 class="chapter-title">第一章 山门</h1>
                        <p>第一段。</p>
                        <p>第二段。</p>
                    </article>
                    <a class="next" href="/book/2">下一章</a>
                </body>
            </html>
        "#;

        let page = extractor
            .extract_page(
                1,
                &Url::parse("https://example.com/book/1").expect("valid url"),
                html,
            )
            .expect("page extracts");

        assert_eq!(page.title, "第一章 山门");
        assert_eq!(page.body, "第一段。\n\n第二段。");
        assert_eq!(
            page.next_url.expect("has next page").as_str(),
            "https://example.com/book/2"
        );
    }

    #[test]
    fn infer_next_url_should_understand_common_chinese_label() {
        let document = Html::parse_document(
            r#"<html><body><a href="/chapter-2">上一章</a><a href="/chapter-3">下一章</a></body></html>"#,
        );
        let url = Url::parse("https://example.com/chapter-1").expect("valid url");

        let next = infer_next_url(&document, &url)
            .expect("inference succeeds")
            .expect("next link found");

        assert_eq!(next.as_str(), "https://example.com/chapter-3");
    }
}
