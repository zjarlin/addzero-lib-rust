// Inline live-preview example — Obsidian Notes style.
//
// All blocks render as formatted HTML. The block under the cursor reverts to
// raw markdown so you type directly into the source. Moving the cursor to
// another block re-renders the previous block back to HTML.
//
// Uses `LivePreviewVariant::Inline` — no split pane, no Divider, no Preview.

use dioxus::prelude::*;
use dioxus_nox_markdown::markdown;
use dioxus_nox_markdown::prelude::{
    LivePreviewVariant, MarkdownHandle, Mode, use_heading_index, use_markdown_handle,
};

fn main() {
    dioxus::launch(App);
}

const SAMPLE: &str = "\
# Inline Live Preview

This editor uses **Obsidian-style** inline live preview.

## How It Works

- All blocks render as formatted HTML
- Click a block to edit it as raw markdown
- Move away to re-render as formatted HTML

## Try It

Edit this *paragraph* directly. Notice that `**bold**` and `_italic_` markers
appear only when the cursor is inside this block.

### Code Block

```rust
fn hello() {
    println!(\"Hello, inline preview!\");
}
```

### Quote

> Inline preview collapses the editor/preview split into a single surface.

## More Content

Add more paragraphs to see each block independently switch between rendered
and raw modes as you move your cursor around the document.
";

/// Inline live-preview example demonstrating cursor-aware block switching.
///
/// Key differences from the split-pane live preview:
/// - `live_preview_variant: LivePreviewVariant::Inline` on Root
/// - No `markdown::Divider` or `markdown::Preview` (they render nothing in inline mode)
/// - `markdown::Editor` renders a `<div contenteditable>` managed via eval()
#[component]
fn App() -> Element {
    let mut current_mode = use_signal(|| Mode::LivePreview);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/examples.css") }
        markdown::Root {
            mode: current_mode,
            on_mode_change: move |m: Mode| current_mode.set(m),
            default_value: SAMPLE,
            live_preview_variant: LivePreviewVariant::Inline,

            // ── Mode bar ──────────────────────────────────────────
            markdown::ModeBar {
                markdown::ModeTab { mode: Mode::Read, "Read" }
                markdown::ModeTab { mode: Mode::Source, "Source" }
                markdown::ModeTab { mode: Mode::LivePreview, "Inline Preview" }
            }

            // ── Toolbar ───────────────────────────────────────────
            markdown::Toolbar {
                FormatToolbar {}
            }

            // ── Inline editor — no Divider or Preview needed ──────
            markdown::Editor {
                placeholder: "Type markdown here...",
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
