use std::collections::HashSet;

use anyhow::Result;
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

use crate::web::{
    first_non_empty_text, href_from_element, normalized_text, parse_selector, same_origin,
};

use super::preset::ResolvedSelectors;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub title: Option<String>,
    pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedToc {
    pub book_title: String,
    pub chapters: Vec<TocEntry>,
}

#[derive(Debug)]
pub struct TocParser {
    book_title_selectors: Vec<Selector>,
    chapter_list_selectors: Vec<Selector>,
    chapter_link_selectors: Vec<Selector>,
    fallback_h1_selector: Selector,
    fallback_title_selector: Selector,
}

impl TocParser {
    pub fn from_resolved(selectors: &ResolvedSelectors) -> Result<Self> {
        let book_title_selectors = selectors
            .book_title_selectors
            .iter()
            .map(|value| parse_selector(value, "book title selector"))
            .collect::<Result<Vec<_>>>()?;
        let chapter_list_selectors = selectors
            .chapter_list_selectors
            .iter()
            .map(|value| parse_selector(value, "chapter list selector"))
            .collect::<Result<Vec<_>>>()?;
        let chapter_link_selectors = selectors
            .chapter_link_selectors
            .iter()
            .map(|value| parse_selector(value, "chapter link selector"))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            book_title_selectors,
            chapter_list_selectors,
            chapter_link_selectors,
            fallback_h1_selector: parse_selector("h1", "fallback h1 selector")?,
            fallback_title_selector: parse_selector("title", "fallback title selector")?,
        })
    }

    pub fn parse(
        &self,
        html: &str,
        base_url: &Url,
        output_stem: &str,
        allow_offsite: bool,
    ) -> Result<ParsedToc> {
        let document = Html::parse_document(html);
        let book_title = self
            .extract_book_title(&document)
            .unwrap_or_else(|| output_stem.to_owned());
        let chapters = self.extract_chapters(&document, base_url, allow_offsite);

        if chapters.is_empty() {
            anyhow::bail!("no chapter links matched in TOC page");
        }

        Ok(ParsedToc {
            book_title,
            chapters,
        })
    }

    fn extract_book_title(&self, document: &Html) -> Option<String> {
        for selector in &self.book_title_selectors {
            if let Some(title) = first_non_empty_text(document, selector) {
                return Some(title);
            }
        }

        first_non_empty_text(document, &self.fallback_h1_selector)
            .or_else(|| first_non_empty_text(document, &self.fallback_title_selector))
    }

    fn extract_chapters(
        &self,
        document: &Html,
        base_url: &Url,
        allow_offsite: bool,
    ) -> Vec<TocEntry> {
        let mut seen = HashSet::new();
        let mut chapters = Vec::new();

        if self.chapter_list_selectors.is_empty() {
            self.collect_links_from_document(
                document,
                base_url,
                allow_offsite,
                &mut seen,
                &mut chapters,
            );
            return chapters;
        }

        for list_selector in &self.chapter_list_selectors {
            for scope in document.select(list_selector) {
                self.collect_links_from_scope(
                    scope,
                    base_url,
                    allow_offsite,
                    &mut seen,
                    &mut chapters,
                );
            }
        }

        chapters
    }

    fn collect_links_from_document(
        &self,
        document: &Html,
        base_url: &Url,
        allow_offsite: bool,
        seen: &mut HashSet<String>,
        chapters: &mut Vec<TocEntry>,
    ) {
        for link_selector in &self.chapter_link_selectors {
            for link in document.select(link_selector) {
                self.push_link(link, base_url, allow_offsite, seen, chapters);
            }
        }
    }

    fn collect_links_from_scope(
        &self,
        scope: ElementRef<'_>,
        base_url: &Url,
        allow_offsite: bool,
        seen: &mut HashSet<String>,
        chapters: &mut Vec<TocEntry>,
    ) {
        for link_selector in &self.chapter_link_selectors {
            for link in scope.select(link_selector) {
                self.push_link(link, base_url, allow_offsite, seen, chapters);
            }
        }
    }

    fn push_link(
        &self,
        link: ElementRef<'_>,
        base_url: &Url,
        allow_offsite: bool,
        seen: &mut HashSet<String>,
        chapters: &mut Vec<TocEntry>,
    ) {
        let Some(url) = href_from_element(base_url, &link) else {
            return;
        };
        if !allow_offsite && !same_origin(base_url, &url) {
            return;
        }

        let key = url.as_str().to_owned();
        if !seen.insert(key) {
            return;
        }

        chapters.push(TocEntry {
            title: normalized_text(&link),
            url,
        });
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use super::*;
    use crate::novel::NovelFetchConfig;
    use crate::novel::preset::{NovelPreset, resolve_selectors};
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn parser_understands_biqukan_style_lists_and_deduplicates_urls() {
        let parser = parser_for(NovelPreset::Biqukan);
        let html = r#"
            <html>
                <body>
                    <div class="book"><div class="info"><h2>诡秘之主</h2></div></div>
                    <div class="listmain">
                        <dl>
                            <dd><a href="/15_15338/1.html">第一章 黑荆棘</a></dd>
                            <dd><a href="/15_15338/2.html">第二章 占卜家</a></dd>
                            <dd><a href="/15_15338/1.html">重复</a></dd>
                        </dl>
                    </div>
                </body>
            </html>
        "#;

        let toc = parser
            .parse(
                html,
                &Url::parse("https://www.biqukan.com/15_15338/").expect("url"),
                "fallback",
                false,
            )
            .expect("toc parses");

        assert_eq!(toc.book_title, "诡秘之主");
        assert_eq!(toc.chapters.len(), 2);
        assert_eq!(toc.chapters[0].title.as_deref(), Some("第一章 黑荆棘"));
    }

    #[test]
    fn parser_uses_h1_then_title_then_output_stem_for_book_title() {
        let parser = parser_for(NovelPreset::Custom);
        let url = Url::parse("https://example.com/book/").expect("url");

        let h1 = parser
            .parse(
                "<html><body><h1>目录标题</h1><a href=\"/1\">第一章</a></body></html>",
                &url,
                "fallback",
                false,
            )
            .expect("toc parses");
        assert_eq!(h1.book_title, "目录标题");

        let title = parser
            .parse(
                "<html><head><title>页面标题</title></head><body><a href=\"/1\">第一章</a></body></html>",
                &url,
                "fallback",
                false,
            )
            .expect("toc parses");
        assert_eq!(title.book_title, "页面标题");

        let stem = parser
            .parse(
                "<html><body><a href=\"/1\">第一章</a></body></html>",
                &url,
                "fallback",
                false,
            )
            .expect("toc parses");
        assert_eq!(stem.book_title, "fallback");
    }

    #[test]
    fn parser_understands_xbqg_style_lists() {
        let parser = parser_for(NovelPreset::Xbqg);
        let html = r#"
            <html>
                <body>
                    <div id="info"><h1>道诡异仙</h1></div>
                    <div id="list">
                        <a href="/12_12000/1.html">第一章 入门</a>
                        <a href="/12_12000/2.html">第二章 山路</a>
                    </div>
                </body>
            </html>
        "#;

        let toc = parser
            .parse(
                html,
                &Url::parse("https://www.xsbiquge.com/12_12000/").expect("url"),
                "fallback",
                false,
            )
            .expect("toc parses");

        assert_eq!(toc.book_title, "道诡异仙");
        assert_eq!(toc.chapters.len(), 2);
        assert_eq!(
            toc.chapters[1].url.as_str(),
            "https://www.xsbiquge.com/12_12000/2.html"
        );
    }

    #[test]
    fn parser_filters_offsite_links_unless_enabled() {
        let parser = parser_for(NovelPreset::Custom);
        let html = r#"
            <html>
                <body>
                    <a href="/1">站内</a>
                    <a href="https://other.example.com/2">站外</a>
                </body>
            </html>
        "#;
        let url = Url::parse("https://example.com/book/").expect("url");

        let onsite_only = parser
            .parse(html, &url, "fallback", false)
            .expect("toc parses");
        assert_eq!(onsite_only.chapters.len(), 1);
        assert_eq!(onsite_only.chapters[0].title.as_deref(), Some("站内"));

        let allow_offsite = parser
            .parse(html, &url, "fallback", true)
            .expect("toc parses");
        assert_eq!(allow_offsite.chapters.len(), 2);
    }

    fn parser_for(preset: NovelPreset) -> TocParser {
        let config = NovelFetchConfig {
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
        };
        let selectors = resolve_selectors(&config).expect("selectors resolve");
        TocParser::from_resolved(&selectors).expect("parser builds")
    }
}
