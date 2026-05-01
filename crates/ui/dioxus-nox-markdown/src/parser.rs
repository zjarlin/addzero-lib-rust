use crop::Rope;
use dioxus::prelude::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::ops::Range;

use crate::types::{HeadingEntry, HtmlRenderPolicy, NodeType, OwnedAstNode, ParsedDoc};

/// Configuration for rendering the AST to Dioxus elements.
#[derive(Debug, Clone)]
pub(crate) struct RenderConfig<'a> {
    pub html_render_policy: HtmlRenderPolicy,
    pub highlight_class_prefix: &'a str,
    pub show_code_line_numbers: bool,
    pub show_code_language: bool,
}

pub enum CustomEvent<'a> {
    Standard(Event<'a>),
    Wikilink(String),
    Tag(String),
}

/// Build pulldown-cmark options with GFM extensions enabled.
pub(crate) fn build_cmark_options() -> Options {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    opts
}

/// A node in our custom AST, mapped directly to byte ranges in the Rope.
pub struct AstNode<'a> {
    pub event: CustomEvent<'a>,
    pub range: Range<usize>,
    pub children: Vec<AstNode<'a>>,
}

/// Parse a markdown rope into a `ParsedDoc`.
///
/// Builds a hierarchical AST using pulldown-cmark events, preserving exact byte
/// offsets for the LivePreview mapping.
pub fn parse_document(input: &Rope) -> ParsedDoc {
    parse_document_with_policy(input, HtmlRenderPolicy::Escape)
}

/// Parse a markdown rope into a `ParsedDoc` with explicit HTML render policy.
pub fn parse_document_with_policy(input: &Rope, html_render_policy: HtmlRenderPolicy) -> ParsedDoc {
    parse_document_full(input, html_render_policy, "hl-")
}

/// Parse a markdown rope with explicit HTML render policy and highlight class prefix.
///
/// `highlight_class_prefix` is prepended to CSS class names in syntax-highlighted
/// code blocks (e.g. `"hl-"` produces `<span class="hl-keyword">`).
pub fn parse_document_full(
    input: &Rope,
    html_render_policy: HtmlRenderPolicy,
    highlight_class_prefix: &str,
) -> ParsedDoc {
    parse_document_full_with_config(
        input,
        html_render_policy,
        highlight_class_prefix,
        false,
        true,
    )
}

/// Parse a markdown rope with full configuration (including code block display options).
pub fn parse_document_full_with_config(
    input: &Rope,
    html_render_policy: HtmlRenderPolicy,
    highlight_class_prefix: &str,
    show_code_line_numbers: bool,
    show_code_language: bool,
) -> ParsedDoc {
    let text = input.to_string(); // Temporary string for parsing, rope backing to come next if needed
    let opts = build_cmark_options();
    let parser = Parser::new_ext(&text, opts).into_offset_iter();

    let mut root_children = Vec::new();
    let mut stack: Vec<AstNode> = Vec::new();

    let mut headings = Vec::new();
    let mut front_matter = None;

    // A simple builder to convert the flat event stream into a tree
    for (event, range) in parser {
        match event {
            Event::Start(_) => {
                stack.push(AstNode {
                    event: CustomEvent::Standard(event),
                    range,
                    children: Vec::new(),
                });
            }
            Event::End(_tag_end) => {
                let mut node = stack.pop().expect("Mismatched End event in parser");
                // Update the range to encompass the entire node
                node.range.end = range.end;

                // Track headings
                if let CustomEvent::Standard(Event::Start(Tag::Heading { level, .. })) = &node.event
                {
                    let text_content = extract_text(&node);

                    let mut anchor = String::new();
                    let mut last_was_dash = false;
                    for c in text_content.to_lowercase().chars() {
                        if c.is_alphanumeric() {
                            anchor.push(c);
                            last_was_dash = false;
                        } else if !last_was_dash {
                            anchor.push('-');
                            last_was_dash = true;
                        }
                    }
                    let anchor = anchor.trim_matches('-').to_string();

                    headings.push(HeadingEntry {
                        level: *level as u8,
                        text: text_content.clone(),
                        anchor,
                        line: index_to_line_col(&text, node.range.start).0,
                    });
                } else if let CustomEvent::Standard(Event::Start(Tag::MetadataBlock(_))) =
                    &node.event
                {
                    front_matter = Some(text[node.range.clone()].to_string());
                }

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root_children.push(node);
                }
            }
            _ => {
                let node = AstNode {
                    event: CustomEvent::Standard(event),
                    range,
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root_children.push(node);
                }
            }
        }
    }

    // Pass 2: Replace standard text nodes with custom Wikilink and Tag nodes
    second_pass_custom_extensions(&mut root_children, &text);

    let config = RenderConfig {
        html_render_policy,
        highlight_class_prefix,
        show_code_line_numbers,
        show_code_language,
    };
    let element = render_ast_to_element(&root_children, &config);
    let ast = root_children.iter().filter_map(to_owned_node).collect();

    ParsedDoc {
        element,
        headings,
        front_matter,
        blocks: Vec::new(), // Inline Blocks to be completely removed/rewritten
        ast,
    }
}

fn to_owned_node(node: &AstNode) -> Option<OwnedAstNode> {
    let node_type = match &node.event {
        CustomEvent::Standard(Event::Start(tag)) => match tag {
            Tag::Paragraph => NodeType::Paragraph,
            Tag::Heading { level, .. } => NodeType::Heading(*level as u8),
            Tag::BlockQuote(_) => NodeType::BlockQuote,
            Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(info)) => {
                let lang = info.split_whitespace().next().unwrap_or("").to_string();
                NodeType::CodeBlock(lang)
            }
            Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Indented) => {
                NodeType::CodeBlock(String::new())
            }
            Tag::List(start) => NodeType::List(*start),
            Tag::Item => NodeType::Item,
            Tag::Emphasis => NodeType::Emphasis,
            Tag::Strong => NodeType::Strong,
            Tag::Strikethrough => NodeType::Strikethrough,
            Tag::Link {
                dest_url, title, ..
            } => NodeType::Link {
                url: dest_url.to_string(),
                title: title.to_string(),
            },
            Tag::Image {
                dest_url, title, ..
            } => NodeType::Image {
                url: dest_url.to_string(),
                title: title.to_string(),
            },
            Tag::Table(_) => NodeType::Table,
            Tag::TableHead => NodeType::TableHead,
            Tag::TableRow => NodeType::TableRow,
            Tag::TableCell => NodeType::TableCell,
            Tag::FootnoteDefinition(s) => NodeType::FootnoteReference(s.to_string()),
            Tag::HtmlBlock => NodeType::HtmlBlock,
            Tag::MetadataBlock(_) => NodeType::Rule,
            Tag::DefinitionList => NodeType::DefinitionList,
            Tag::DefinitionListTitle => NodeType::DefinitionListTitle,
            Tag::DefinitionListDefinition => NodeType::DefinitionListDefinition,
            Tag::Superscript => NodeType::Superscript,
            Tag::Subscript => NodeType::Subscript,
        },
        CustomEvent::Standard(Event::Text(t)) => NodeType::Text(t.to_string()),
        CustomEvent::Standard(Event::Code(c)) => NodeType::Code(c.to_string()),
        CustomEvent::Standard(Event::Html(h)) | CustomEvent::Standard(Event::InlineHtml(h)) => {
            NodeType::Html(h.to_string())
        }
        CustomEvent::Standard(Event::SoftBreak) => NodeType::SoftBreak,
        CustomEvent::Standard(Event::HardBreak) => NodeType::HardBreak,
        CustomEvent::Standard(Event::Rule) => NodeType::Rule,
        CustomEvent::Standard(Event::TaskListMarker(b)) => NodeType::TaskListMarker(*b),
        CustomEvent::Standard(Event::FootnoteReference(f)) => {
            NodeType::FootnoteReference(f.to_string())
        }
        CustomEvent::Wikilink(link) => NodeType::Wikilink(link.clone()),
        CustomEvent::Tag(tag) => NodeType::Tag(tag.clone()),
        _ => return None,
    };

    Some(OwnedAstNode {
        node_type,
        range: node.range.clone(),
        children: node.children.iter().filter_map(to_owned_node).collect(),
    })
}

fn extract_text(node: &AstNode) -> String {
    let mut buf = String::new();
    for child in &node.children {
        match &child.event {
            CustomEvent::Standard(Event::Text(t)) => buf.push_str(t),
            CustomEvent::Standard(Event::Code(c)) => buf.push_str(c),
            CustomEvent::Wikilink(link) => buf.push_str(link),
            CustomEvent::Tag(tag) => buf.push_str(tag),
            _ => buf.push_str(&extract_text(child)),
        }
    }
    buf
}

/// Convert a byte offset into (line, column) for a given text.
/// Both line and column are 0-based.
pub fn index_to_line_col(text: &str, index: usize) -> (usize, usize) {
    let before = &text[..index];
    let line = before.bytes().filter(|&b| b == b'\n').count();
    let col = match before.rfind('\n') {
        Some(nl_pos) => index - nl_pos - 1,
        None => index,
    };
    (line, col)
}

/// Helper: sanitize href strings.
pub(crate) fn sanitize_href(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }
    if trimmed.starts_with('#') || trimmed.starts_with('/') {
        return Some(trimmed.to_string());
    }
    let lower = trimmed.to_lowercase();
    if let Some(colon_pos) = lower.find(':') {
        let scheme = &lower[..colon_pos];
        match scheme {
            "javascript" | "data" | "vbscript" => return None,
            _ => return Some(trimmed.to_string()),
        }
    }
    Some(trimmed.to_string())
}

/// Render the AST tree into a Dioxus Element.
pub(crate) fn render_ast_to_element(children: &[AstNode], config: &RenderConfig) -> Element {
    let kids: Vec<Element> = children
        .iter()
        .map(|node| render_node(node, config))
        .collect();
    rsx! {
        for child in kids {
            {child}
        }
    }
}

fn render_node(node: &AstNode, config: &RenderConfig) -> Element {
    let start_str = node.range.start.to_string();
    let end_str = node.range.end.to_string();

    match &node.event {
        CustomEvent::Standard(Event::Start(tag)) => {
            render_tag(tag, &node.children, node.range.clone(), config)
        }
        CustomEvent::Standard(Event::Text(t)) => {
            let txt = t.to_string();
            rsx! { "{txt}" }
        }
        CustomEvent::Standard(Event::Code(c)) => {
            let code = c.to_string();
            rsx! { code { "{code}" } }
        }
        CustomEvent::Standard(Event::SoftBreak) => rsx! { " " },
        CustomEvent::Standard(Event::HardBreak) => rsx! { br {} },
        CustomEvent::Standard(Event::Rule) => {
            rsx! { hr { "data-source-start": "{start_str}", "data-source-end": "{end_str}" } }
        }
        CustomEvent::Standard(Event::Html(h)) | CustomEvent::Standard(Event::InlineHtml(h)) => {
            let html = h.to_string();
            match config.html_render_policy {
                HtmlRenderPolicy::Trusted => {
                    rsx! { span { dangerous_inner_html: "{html}" } }
                }
                #[cfg(feature = "sanitize")]
                HtmlRenderPolicy::Sanitized => {
                    let clean = ammonia::clean(&html);
                    rsx! { span { dangerous_inner_html: "{clean}" } }
                }
                // Escape (default) — and Sanitized fallback when feature is disabled
                _ => {
                    rsx! { span { "{html}" } }
                }
            }
        }
        CustomEvent::Standard(Event::TaskListMarker(checked)) => {
            let is_checked = *checked;
            rsx! { input { r#type: "checkbox", checked: is_checked, disabled: true } }
        }
        CustomEvent::Standard(Event::FootnoteReference(f)) => {
            let f_str = f.to_string();
            rsx! { sup { a { href: "#fn-{f_str}", "data-md-footnote-ref": "{f_str}", "{f_str}" } } }
        }
        CustomEvent::Tag(tag) => {
            rsx! { span { "data-md-tag": "{tag}", "data-source-start": "{start_str}", "data-source-end": "{end_str}", "{tag}" } }
        }
        CustomEvent::Wikilink(link) => {
            rsx! { a { "data-md-wikilink": "{link}", "data-source-start": "{start_str}", "data-source-end": "{end_str}", "[[{link}]]" } }
        }
        _ => rsx! { span {} },
    }
}

fn render_tag(
    tag: &Tag,
    children: &[AstNode],
    range: Range<usize>,
    config: &RenderConfig,
) -> Element {
    let start_str = range.start.to_string();
    let end_str = range.end.to_string();
    let kids_elements: Vec<Element> = children
        .iter()
        .map(|node| render_node(node, config))
        .collect();
    let kids = rsx! { for c in kids_elements { {c} } };

    match tag {
        Tag::Paragraph => {
            rsx! { p { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::Heading { level, .. } => {
            let l = *level as u8;
            match l {
                1 => {
                    rsx! { h1 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
                2 => {
                    rsx! { h2 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
                3 => {
                    rsx! { h3 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
                4 => {
                    rsx! { h4 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
                5 => {
                    rsx! { h5 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
                _ => {
                    rsx! { h6 { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
                }
            }
        }
        Tag::BlockQuote(_) => {
            rsx! { blockquote { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(info)) => {
            let lang = info.split_whitespace().next().unwrap_or("").to_string();
            let code_text = extract_text_from_children(children);
            let mut result =
                crate::highlight::highlight_code(&code_text, &lang, config.highlight_class_prefix);
            if config.show_code_line_numbers {
                result.html = crate::highlight::wrap_with_line_numbers(&result.html);
            }
            let highlighted_attr = result.language_matched.then_some("true");
            let line_numbers_attr = config.show_code_line_numbers.then_some("");
            let show_lang_header = config.show_code_language && !lang.is_empty();
            rsx! {
                pre {
                    "data-md-code-block": "",
                    "data-md-language": "{lang}",
                    "data-md-highlighted": highlighted_attr,
                    "data-md-line-numbers": line_numbers_attr,
                    "data-source-start": "{start_str}",
                    "data-source-end": "{end_str}",
                    if show_lang_header {
                        div {
                            "data-md-code-header": "",
                            span { "data-md-code-language": "", "{lang}" }
                        }
                    }
                    code {
                        class: "language-{lang}",
                        dangerous_inner_html: "{result.html}"
                    }
                }
            }
        }
        Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Indented) => {
            let code_text = extract_text_from_children(children);
            let mut result =
                crate::highlight::highlight_code(&code_text, "", config.highlight_class_prefix);
            if config.show_code_line_numbers {
                result.html = crate::highlight::wrap_with_line_numbers(&result.html);
            }
            let line_numbers_attr = config.show_code_line_numbers.then_some("");
            rsx! {
                pre {
                    "data-md-code-block": "",
                    "data-md-line-numbers": line_numbers_attr,
                    "data-source-start": "{start_str}",
                    "data-source-end": "{end_str}",
                    code {
                        dangerous_inner_html: "{result.html}"
                    }
                }
            }
        }
        Tag::List(Some(start_idx)) => {
            let idx = *start_idx as i64;
            rsx! { ol { start: "{idx}", "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::List(None) => {
            rsx! { ul { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::Item => {
            // TaskListMarker handling: check if the first child is a TaskListMarker.
            let is_task = children.first().is_some_and(|c| {
                matches!(c.event, CustomEvent::Standard(Event::TaskListMarker(_)))
            });
            if is_task {
                let checked_str = match &children.first().unwrap().event {
                    CustomEvent::Standard(Event::TaskListMarker(checked)) => {
                        if *checked {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    _ => "false",
                };
                rsx! {
                    li {
                        "data-md-task-item": "",
                        "data-md-task-checked": "{checked_str}",
                        "data-source-start": "{start_str}",
                        "data-source-end": "{end_str}",
                        {kids}
                    }
                }
            } else {
                rsx! { li { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
            }
        }
        Tag::Emphasis => {
            rsx! { em { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::Strong => {
            rsx! { strong { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::Strikethrough => {
            rsx! { del { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::Link {
            dest_url, title, ..
        } => {
            let safe_url = sanitize_href(dest_url).unwrap_or_default();
            let title_str = title.to_string();
            let is_external = safe_url.starts_with("http://") || safe_url.starts_with("https://");
            let external_str = if is_external { "true" } else { "false" };
            rsx! {
                a {
                    href: "{safe_url}",
                    title: "{title_str}",
                    "data-md-link": "",
                    "data-md-link-external": "{external_str}",
                    "data-source-start": "{start_str}",
                    "data-source-end": "{end_str}",
                    {kids}
                }
            }
        }
        Tag::Image {
            dest_url, title, ..
        } => {
            let safe_url = sanitize_href(dest_url).unwrap_or_default();
            let title_str = title.to_string();
            // Alt text is rendered by the kids, but for an img tag we need plaintext alt
            let alt_str = extract_text_from_children(children);
            rsx! {
                img {
                    src: "{safe_url}",
                    alt: "{alt_str}",
                    title: "{title_str}",
                    "data-source-start": "{start_str}",
                    "data-source-end": "{end_str}"
                }
            }
        }
        Tag::Table(_) => rsx! {
            div {
                "data-md-table-wrapper": "",
                "data-source-start": "{start_str}",
                "data-source-end": "{end_str}",
                table { {kids} }
            }
        },
        Tag::TableHead => rsx! { thead { tr { {kids} } } },
        Tag::TableRow => rsx! { tr { {kids} } },
        Tag::TableCell => rsx! { td { {kids} } },
        Tag::FootnoteDefinition(name) => {
            let name_str = name.to_string();
            rsx! {
                div {
                    "data-md-footnote-def": "{name_str}",
                    "data-source-start": "{start_str}",
                    "data-source-end": "{end_str}",
                    {kids}
                }
            }
        }
        Tag::HtmlBlock => {
            rsx! { div { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
        Tag::MetadataBlock(_) => rsx! { div { display: "none" } },
        _ => {
            rsx! { span { "data-source-start": "{start_str}", "data-source-end": "{end_str}", {kids} } }
        }
    }
}

fn extract_text_from_children(children: &[AstNode]) -> String {
    let mut buf = String::new();
    for child in children {
        if let CustomEvent::Standard(Event::Text(t)) = &child.event {
            buf.push_str(t);
        } else {
            buf.push_str(&extract_text_from_children(&child.children));
        }
    }
    buf
}

fn second_pass_custom_extensions<'a>(nodes: &mut Vec<AstNode<'a>>, _text_source: &str) {
    // Pre-process: merge adjacent Text nodes because pulldown_cmark can fragment failed reference links like `[[`
    let mut merged: Vec<AstNode<'a>> = Vec::new();
    for node in nodes.drain(..) {
        if let CustomEvent::Standard(Event::Text(ref t)) = node.event
            && let Some(last) = merged.last_mut()
            && let CustomEvent::Standard(Event::Text(ref mut last_t)) = last.event
        {
            let mut combined = last_t.to_string();
            combined.push_str(t);
            last.event = CustomEvent::Standard(Event::Text(combined.into()));
            last.range.end = node.range.end;
            continue;
        }
        merged.push(node);
    }
    *nodes = merged;

    let mut new_nodes = Vec::new();
    let mut replaced = false;

    for node in nodes.drain(..) {
        let maybe_text = match &node.event {
            CustomEvent::Standard(Event::Text(text)) => Some(text.to_string()),
            _ => None,
        };

        if let Some(text) = maybe_text {
            // Find hashtags and wikilinks inside the text node with native scanning.
            let mut matches = scan_custom_tokens(&text);

            if matches.is_empty() {
                new_nodes.push(node);
                continue;
            }

            replaced = true;
            matches.sort_by_key(|a| a.0);

            let mut last_idx = 0;
            let start_offset = node.range.start;

            for (m_start, m_end, token) in matches {
                if m_start > last_idx {
                    // Push standard text node before the match
                    let slice = &text[last_idx..m_start];
                    new_nodes.push(AstNode {
                        event: CustomEvent::Standard(Event::Text(slice.to_string().into())),
                        range: (start_offset + last_idx)..(start_offset + m_start),
                        children: Vec::new(),
                    });
                }

                // Push custom event
                let event = match token {
                    TokenKind::Tag(tag) => CustomEvent::Tag(tag),
                    TokenKind::Wikilink(link) => CustomEvent::Wikilink(link),
                };
                new_nodes.push(AstNode {
                    event,
                    range: (start_offset + m_start)..(start_offset + m_end),
                    children: Vec::new(),
                });

                last_idx = m_end;
            }

            // Push remainder
            if last_idx < text.len() {
                let slice = &text[last_idx..text.len()];
                new_nodes.push(AstNode {
                    event: CustomEvent::Standard(Event::Text(slice.to_string().into())),
                    range: (start_offset + last_idx)..(start_offset + text.len()),
                    children: Vec::new(),
                });
            }
        } else {
            // Recurse heavily into children
            let mut recursed_node = node;
            second_pass_custom_extensions(&mut recursed_node.children, _text_source);
            new_nodes.push(recursed_node);
        }
    }

    if replaced || !new_nodes.is_empty() {
        *nodes = new_nodes;
    }
}

enum TokenKind {
    Tag(String),
    Wikilink(String),
}

fn scan_custom_tokens(text: &str) -> Vec<(usize, usize, TokenKind)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        // Wikilink: [[...]]
        if bytes[i] == b'[' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            let mut found = None;
            while j + 1 < bytes.len() {
                if bytes[j] == b']' && bytes[j + 1] == b']' {
                    found = Some(j);
                    break;
                }
                j += 1;
            }
            if let Some(end_open) = found {
                let inner = &text[i + 2..end_open];
                out.push((i, end_open + 2, TokenKind::Wikilink(inner.to_string())));
                i = end_open + 2;
                continue;
            }
        }

        // Tag: #[A-Za-z0-9_-]+
        if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() {
                let b = bytes[j];
                let valid = (b as char).is_ascii_alphanumeric() || b == b'_' || b == b'-';
                if valid {
                    j += 1;
                } else {
                    break;
                }
            }
            if j > i + 1 {
                out.push((i, j, TokenKind::Tag(text[i..j].to_string())));
                i = j;
                continue;
            }
        }

        let ch_len = text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        i += ch_len;
    }

    out
}
