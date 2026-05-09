use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use reqwest::Url;

use crate::novel::{NovelFetchConfig, NovelPreset, run_fetch};
use crate::pipeline::{PipelineConfig, TtsMode, run_pipeline};
use crate::web_text::{DownloadConfig, run_download};

#[derive(Debug, Parser)]
#[command(
    name = "addzdero-cli",
    version,
    about = "Novel tooling for video rendering and respectful web text capture"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Download a chapter-list novel using built-in site presets or custom selectors.
    Novel(NovelArgs),
    /// One-command novel-to-video pipeline.
    Novel2Video(Novel2VideoArgs),
    /// Download text from pages you are authorized to crawl.
    WebText(WebTextArgs),
}

#[derive(Debug, Parser)]
struct Novel2VideoArgs {
    /// Input plain-text novel file.
    #[arg(long)]
    input: PathBuf,

    /// Output directory.
    #[arg(long)]
    output: PathBuf,

    /// Video title.
    #[arg(long, default_value = "Novel Video")]
    title: String,

    /// Preferred scene length in Chinese characters.
    #[arg(long, default_value_t = 240)]
    scene_chars: usize,

    /// Output width.
    #[arg(long, default_value_t = 1920)]
    width: u32,

    /// Output height.
    #[arg(long, default_value_t = 1080)]
    height: u32,

    /// FPS.
    #[arg(long, default_value_t = 30)]
    fps: u32,

    /// TTS mode: none | system-say | command
    #[arg(long, default_value = "none")]
    tts: String,

    /// TTS command template when --tts=command.
    /// Placeholders: {input} {output}
    #[arg(long)]
    tts_cmd: Option<String>,

    /// Optional background music file.
    #[arg(long)]
    bgm: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct NovelArgs {
    #[command(subcommand)]
    command: NovelCommand,
}

#[derive(Debug, Subcommand)]
enum NovelCommand {
    /// Download a TOC page and all matching chapter pages into a single text file.
    Fetch(NovelFetchArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum NovelPresetArg {
    Biqukan,
    Xbqg,
    Custom,
}

#[derive(Debug, Args)]
struct NovelFetchArgs {
    /// TOC page that lists all chapter links.
    #[arg(long = "toc-url")]
    toc_url: Url,

    /// Output text file.
    #[arg(long)]
    output: PathBuf,

    /// Built-in selector preset.
    #[arg(long, value_enum)]
    preset: NovelPresetArg,

    /// Optional CSS selector for the book title on the TOC page.
    #[arg(long = "book-title-selector")]
    book_title_selector: Option<String>,

    /// Optional CSS selector that scopes chapter links on the TOC page.
    #[arg(long = "chapter-list-selector")]
    chapter_list_selector: Option<String>,

    /// CSS selector used to find each chapter link on the TOC page.
    #[arg(long = "chapter-link-selector")]
    chapter_link_selector: Option<String>,

    /// Optional CSS selector for the chapter title on each chapter page.
    #[arg(long = "chapter-title-selector")]
    chapter_title_selector: Option<String>,

    /// CSS selector used to extract chapter content.
    /// Repeat this flag to provide fallbacks, first successful selector wins.
    #[arg(long = "content-selector")]
    content_selectors: Vec<String>,

    /// Delay between chapter requests in milliseconds.
    #[arg(long, default_value_t = 800)]
    delay_ms: u64,

    /// User-Agent header to send.
    #[arg(long, default_value = "addzdero-cli/0.1")]
    user_agent: String,

    /// Maximum number of chapters to download.
    #[arg(long)]
    max_chapters: Option<usize>,

    /// Allow following chapter links to a different host.
    #[arg(long, default_value_t = false)]
    allow_offsite: bool,

    /// Skip robots.txt checks.
    #[arg(long, default_value_t = false)]
    ignore_robots: bool,
}

#[derive(Debug, Args)]
struct WebTextArgs {
    #[command(subcommand)]
    command: WebTextCommand,
}

#[derive(Debug, Subcommand)]
enum WebTextCommand {
    /// Follow chapter-style pagination and save extracted text to a local file.
    Fetch(WebTextFetchArgs),
}

#[derive(Debug, Args)]
struct WebTextFetchArgs {
    /// First page to download.
    #[arg(long)]
    url: Url,

    /// Output text file.
    #[arg(long)]
    output: PathBuf,

    /// CSS selector used to extract the main text.
    /// Repeat this flag to provide fallbacks, first successful selector wins.
    #[arg(long = "content-selector", required = true)]
    content_selectors: Vec<String>,

    /// Optional CSS selector for the chapter title.
    #[arg(long = "title-selector")]
    title_selector: Option<String>,

    /// CSS selector used to find the next page link.
    /// Repeat this flag to provide fallbacks.
    #[arg(long = "next-selector")]
    next_selectors: Vec<String>,

    /// Maximum number of pages to follow.
    #[arg(long, default_value_t = 200)]
    max_pages: usize,

    /// Delay between requests in milliseconds.
    #[arg(long, default_value_t = 800)]
    delay_ms: u64,

    /// User-Agent header to send.
    #[arg(long, default_value = "addzdero-cli/0.1")]
    user_agent: String,

    /// Allow following next-page links to a different host.
    #[arg(long, default_value_t = false)]
    allow_offsite: bool,

    /// Skip robots.txt checks.
    #[arg(long, default_value_t = false)]
    ignore_robots: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

impl From<NovelPresetArg> for NovelPreset {
    fn from(value: NovelPresetArg) -> Self {
        match value {
            NovelPresetArg::Biqukan => NovelPreset::Biqukan,
            NovelPresetArg::Xbqg => NovelPreset::Xbqg,
            NovelPresetArg::Custom => NovelPreset::Custom,
        }
    }
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Novel(args) => match args.command {
            NovelCommand::Fetch(args) => {
                let config = NovelFetchConfig {
                    toc_url: args.toc_url,
                    output: args.output,
                    preset: args.preset.into(),
                    book_title_selector: args.book_title_selector,
                    chapter_list_selector: args.chapter_list_selector,
                    chapter_link_selector: args.chapter_link_selector,
                    chapter_title_selector: args.chapter_title_selector,
                    content_selectors: args.content_selectors,
                    delay: Duration::from_millis(args.delay_ms),
                    user_agent: args.user_agent,
                    max_chapters: args.max_chapters,
                    allow_offsite: args.allow_offsite,
                    ignore_robots: args.ignore_robots,
                };

                run_fetch(config)
            }
        },
        Command::Novel2Video(args) => {
            let tts_mode = match args.tts.as_str() {
                "none" => TtsMode::None,
                "system-say" => TtsMode::SystemSay,
                "command" => TtsMode::Command(args.tts_cmd.unwrap_or_default()),
                other => anyhow::bail!("unsupported --tts mode: {other}"),
            };

            if matches!(tts_mode, TtsMode::Command(ref cmd) if cmd.trim().is_empty()) {
                anyhow::bail!("--tts=command requires --tts-cmd");
            }

            let config = PipelineConfig {
                input: args.input,
                output: args.output,
                title: args.title,
                scene_chars: args.scene_chars,
                width: args.width,
                height: args.height,
                fps: args.fps,
                tts_mode,
                bgm: args.bgm,
            };

            run_pipeline(config)
        }
        Command::WebText(args) => match args.command {
            WebTextCommand::Fetch(args) => {
                if args.max_pages == 0 {
                    anyhow::bail!("--max-pages must be greater than 0");
                }

                let config = DownloadConfig {
                    start_url: args.url,
                    output: args.output,
                    content_selectors: args.content_selectors,
                    title_selector: args.title_selector,
                    next_selectors: args.next_selectors,
                    max_pages: args.max_pages,
                    delay: Duration::from_millis(args.delay_ms),
                    user_agent: args.user_agent,
                    allow_offsite: args.allow_offsite,
                    ignore_robots: args.ignore_robots,
                };

                run_download(config)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::*;

    #[test]
    fn clap_name_matches_renamed_binary() {
        assert_eq!(Cli::command().get_name(), "addzdero-cli");
    }

    #[test]
    fn parses_novel_fetch_command() {
        let cli = Cli::try_parse_from([
            "addzdero-cli",
            "novel",
            "fetch",
            "--toc-url",
            "https://example.com/book/",
            "--output",
            "book.txt",
            "--preset",
            "xbqg",
            "--content-selector",
            "#content",
            "--max-chapters",
            "10",
        ])
        .expect("cli parses");

        match cli.command {
            Command::Novel(args) => match args.command {
                NovelCommand::Fetch(args) => {
                    assert_eq!(args.preset, NovelPresetArg::Xbqg);
                    assert_eq!(args.max_chapters, Some(10));
                    assert_eq!(args.content_selectors, vec!["#content"]);
                }
            },
            _ => panic!("expected novel command"),
        }
    }

    #[test]
    fn parses_custom_preset_selector_overrides() {
        let cli = Cli::try_parse_from([
            "addzdero-cli",
            "novel",
            "fetch",
            "--toc-url",
            "https://example.com/book/",
            "--output",
            "book.txt",
            "--preset",
            "custom",
            "--chapter-link-selector",
            "section.toc a",
            "--chapter-title-selector",
            "h2.title",
            "--content-selector",
            "article",
            "--content-selector",
            "#reader",
        ])
        .expect("cli parses");

        match cli.command {
            Command::Novel(args) => match args.command {
                NovelCommand::Fetch(args) => {
                    assert_eq!(args.preset, NovelPresetArg::Custom);
                    assert_eq!(args.chapter_link_selector.as_deref(), Some("section.toc a"));
                    assert_eq!(args.chapter_title_selector.as_deref(), Some("h2.title"));
                    assert_eq!(args.content_selectors, vec!["article", "#reader"]);
                }
            },
            _ => panic!("expected novel command"),
        }
    }

    #[test]
    fn custom_preset_requires_runtime_selectors() {
        let args = NovelFetchArgs {
            toc_url: Url::parse("https://example.com/book/").expect("url"),
            output: PathBuf::from("book.txt"),
            preset: NovelPresetArg::Custom,
            book_title_selector: None,
            chapter_list_selector: None,
            chapter_link_selector: None,
            chapter_title_selector: None,
            content_selectors: vec![],
            delay_ms: 0,
            user_agent: "test-agent".to_owned(),
            max_chapters: None,
            allow_offsite: false,
            ignore_robots: false,
        };

        let config = NovelFetchConfig {
            toc_url: args.toc_url,
            output: args.output,
            preset: args.preset.into(),
            book_title_selector: args.book_title_selector,
            chapter_list_selector: args.chapter_list_selector,
            chapter_link_selector: args.chapter_link_selector,
            chapter_title_selector: args.chapter_title_selector,
            content_selectors: args.content_selectors,
            delay: Duration::from_millis(args.delay_ms),
            user_agent: args.user_agent,
            max_chapters: args.max_chapters,
            allow_offsite: args.allow_offsite,
            ignore_robots: args.ignore_robots,
        };

        assert!(run_fetch(config).is_err());
    }
}
