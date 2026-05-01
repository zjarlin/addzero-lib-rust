// Alternative styling: see examples/tailwind_styled.rs for Tailwind v4 CSS-first approach.

use dioxus::prelude::*;
use dioxus_nox_markdown::markdown;
use dioxus_nox_markdown::prelude::{
    Layout, MarkdownHandle, Mode, use_heading_index, use_markdown_handle,
};

fn main() {
    dioxus::launch(App);
}

const SAMPLE: &str = "\
---
title: Live Preview Demo
---

# Getting Started

This is a **live preview** example for `dioxus-nox-markdown`.

## Features

- Mode bar (Read / Source / LivePreview)
- Toolbar with Bold, Italic, and Link shortcuts
- Heading index sidebar (table of contents)
- Proportional scroll sync between editor and preview

## Usage

Switch modes using the tab bar above. In **Source** or **LivePreview** mode,
try the toolbar shortcuts:

- **Bold**: select text, click Bold
- _Italic_: select text, click Italic
- [Link](https://example.com): select text, click Link

## Section Three

More content to demonstrate scroll sync. Keep adding text to see the
preview update after each keystroke (debounced ~300ms).

### Deep Nesting

Even deeply nested headings appear in the table of contents.
";

/// Full live-preview example demonstrating all Phase 2 features:
/// - Three-mode ModeBar (Read / Source / LivePreview)
/// - Toolbar with Bold/Italic/Link using MarkdownHandle
/// - use_heading_index() heading list in sidebar
/// - Scroll sync active in LivePreview mode
#[component]
fn App() -> Element {
    let mut current_mode = use_signal(|| Mode::LivePreview);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/examples.css") }
        markdown::Root {
            mode: current_mode,
            on_mode_change: move |m: Mode| current_mode.set(m),
            default_value: SAMPLE,
            layout: Layout::Horizontal,

            // ── Mode bar ──────────────────────────────────────────
            markdown::ModeBar {
                markdown::ModeTab { mode: Mode::Read, "Read" }
                markdown::ModeTab { mode: Mode::Source, "Source" }
                markdown::ModeTab { mode: Mode::LivePreview, "Live Preview" }
            }

            // ── Toolbar ───────────────────────────────────────────
            markdown::Toolbar {
                FormatToolbar {}
            }

            // ── Split pane: Editor + Divider + Preview ────────────
            div { class: "split-pane",
                markdown::Editor {
                    placeholder: "Type markdown here...",
                }
                markdown::Divider {}
                markdown::Preview {}
            }

            // ── Sidebar: heading index ────────────────────────────
            HeadingSidebar {}
        }
    }
}

/// Toolbar buttons wired to MarkdownHandle for Bold/Italic/Link.
/// Must be a descendant of markdown::Root to use use_markdown_handle().
#[component]
fn FormatToolbar() -> Element {
    let handle: MarkdownHandle = use_markdown_handle();

    rsx! {
        markdown::ToolbarButton {
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("**", "**").await;
                });
            },
            "Bold"
        }
        markdown::ToolbarSeparator {}
        markdown::ToolbarButton {
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("_", "_").await;
                });
            },
            "Italic"
        }
        markdown::ToolbarSeparator {}
        markdown::ToolbarButton {
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("[", "](url)").await;
                });
            },
            "Link"
        }
    }
}

/// Sidebar showing the heading index (table of contents).
/// Must be a descendant of markdown::Root to use use_heading_index().
#[component]
fn HeadingSidebar() -> Element {
    let headings = use_heading_index();

    rsx! {
        aside {
            h3 { "Contents" }
            ul {
                for h in headings() {
                    li { "H{h.level} {h.text}" }
                }
            }
        }
    }
}
