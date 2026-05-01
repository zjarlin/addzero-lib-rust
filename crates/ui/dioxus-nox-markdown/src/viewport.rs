use std::rc::Rc;

use dioxus::prelude::*;

use crate::context::MarkdownContext;
use crate::types::{NodeType, OwnedAstNode};

/// Recursively collect text content from an `OwnedAstNode` tree.
fn collect_text_from_ast(node: &OwnedAstNode) -> String {
    let mut buf = String::new();
    match &node.node_type {
        NodeType::Text(t) => buf.push_str(t),
        NodeType::Code(c) => buf.push_str(c),
        _ => {
            for child in &node.children {
                buf.push_str(&collect_text_from_ast(child));
            }
        }
    }
    buf
}

/// A custom component override for a specific Markdown block.
#[derive(Clone)]
pub struct BlockOverride {
    pub matches: Rc<dyn Fn(&OwnedAstNode) -> bool>,
    // Store a callback that returns an Element
    pub component: Rc<dyn Fn(OwnedAstNode) -> Element>,
}

impl PartialEq for BlockOverride {
    fn eq(&self, _other: &Self) -> bool {
        // Functions cannot be easily compared; always trigger an update if overrides change.
        false
    }
}

/// The core headless virtual viewport renderer.
/// Iterates over the `ParsedDoc::ast` and renders each node.
#[derive(Props, Clone, PartialEq)]
pub struct EditorViewportProps {
    /// Optional overrides for specific AST nodes.
    #[props(default)]
    pub overrides: Vec<BlockOverride>,
}

#[component]
pub fn EditorViewport(props: EditorViewportProps) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let parsed = (ctx.parsed_doc)();

    rsx! {
        div {
            class: "nox-md-viewport",
            "data-md-viewport": "true",
            // Render the root AST nodes
            for node in parsed.ast.iter() {
                ViewportNode {
                    node: node.clone(),
                    overrides: props.overrides.clone()
                }
            }
        }
    }
}

#[component]
pub fn ViewportNode(node: OwnedAstNode, overrides: Vec<BlockOverride>) -> Element {
    let ctx = use_context::<MarkdownContext>();

    // Check if any override matches this node
    for ov in overrides.iter() {
        if (ov.matches)(&node) {
            return (ov.component)(node.clone());
        }
    }

    // Default rendering recursively based on NodeType
    match &node.node_type {
        NodeType::Paragraph => {
            rsx! { p { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::Heading(level) => match level {
            1 => {
                rsx! { h1 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
            2 => {
                rsx! { h2 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
            3 => {
                rsx! { h3 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
            4 => {
                rsx! { h4 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
            5 => {
                rsx! { h5 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
            _ => {
                rsx! { h6 { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
            }
        },
        NodeType::Text(t) => {
            let txt = t.clone();
            rsx! { "{txt}" }
        }
        NodeType::Code(c) => {
            let code = c.clone();
            rsx! { code { "{code}" } }
        }
        NodeType::SoftBreak => rsx! { " " },
        NodeType::HardBreak => rsx! { br {} },
        NodeType::Html(h) => {
            let html = h.clone();
            rsx! { span { "{html}" } }
        }
        NodeType::Rule => rsx! { hr {} },
        NodeType::Emphasis => {
            rsx! { em { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::Strong => {
            rsx! { strong { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::Strikethrough => {
            rsx! { del { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::BlockQuote => {
            rsx! { blockquote { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::Wikilink(link) => {
            let l = link.clone();
            rsx! { a { "data-md-wikilink": "{l}", "[[{l}]]" } }
        }
        NodeType::Tag(t) => {
            let tag = t.clone();
            rsx! { span { "data-md-tag": "{tag}", "{tag}" } }
        }
        NodeType::CodeBlock(lang) => {
            let l = lang.clone();
            let code_text = collect_text_from_ast(&node);
            let prefix = ctx.highlight_class_prefix.read().clone();
            let mut result = crate::highlight::highlight_code(&code_text, &l, &prefix);
            if ctx.show_code_line_numbers {
                result.html = crate::highlight::wrap_with_line_numbers(&result.html);
            }
            let highlighted_attr = result.language_matched.then_some("true");
            let line_numbers_attr = ctx.show_code_line_numbers.then_some("");
            let show_lang_header = ctx.show_code_language && !l.is_empty();
            rsx! {
                pre {
                    "data-md-code-block": "",
                    "data-md-language": "{l}",
                    "data-md-highlighted": highlighted_attr,
                    "data-md-line-numbers": line_numbers_attr,
                    if show_lang_header {
                        div {
                            "data-md-code-header": "",
                            span { "data-md-code-language": "", "{l}" }
                        }
                    }
                    code {
                        class: "language-{l}",
                        dangerous_inner_html: "{result.html}"
                    }
                }
            }
        }
        NodeType::List(_) => {
            rsx! { ul { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        NodeType::Item => {
            rsx! { li { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
        _ => {
            rsx! { span { for c in node.children { ViewportNode { node: c, overrides: overrides.clone() } } } }
        }
    }
}
