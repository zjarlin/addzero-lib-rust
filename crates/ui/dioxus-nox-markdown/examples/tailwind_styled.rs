//! # Tailwind v4 CSS-First Styling Example
//!
//! This example demonstrates how to style `dioxus-nox-markdown` components
//! using Tailwind v4's CSS-first approach with utility classes.
//!
//! ## Setup
//!
//! 1. Add Tailwind v4 to your project:
//!    ```
//!    npx @tailwindcss/cli@next -i ./input.css -o ./assets/tailwind.css --watch
//!    ```
//!
//! 2. Link your tailwind.css in the component (shown below).
//!
//! 3. Pass Tailwind utility classes via the `class` prop on each component part.
//!    The components are headless — they apply your classes to their root DOM element.
//!
//! ## Running
//!
//! ```
//! dx serve --package dioxus-nox-markdown --example tailwind_styled
//! ```
//!
//! Alternative: for dark-theme data-attribute based styling see `examples.css`
//! and the `live_preview` example.

// Alternative styling: see crates/markdown/assets/examples.css for data-attribute based approach.

use dioxus::prelude::*;
use dioxus_nox_markdown::markdown;
use dioxus_nox_markdown::prelude::{
    Layout, MarkdownHandle, Mode, use_heading_index, use_markdown_handle,
};

const INITIAL_CONTENT: &str = r#"# Tailwind Styled Editor

This editor is styled with **Tailwind v4** utility classes.

## Features

- Headless components — bring your own styles
- `class` prop on every part
- Works with any CSS framework

> Pass classes via `class: "..."` on each component part.

## Code Example

```rust
rsx! {
    markdown::Root {
        class: "your-tailwind-classes",
        markdown::Editor {}
        markdown::Preview {}
    }
}
```

### Deep Heading

More content for scroll sync demonstration.
"#;

fn main() {
    dioxus::launch(App);
}

/// Full Tailwind-styled example demonstrating all component parts with utility classes.
///
/// Demonstrates:
/// - Tailwind utility classes via the `class` prop on every component part
/// - Three-mode ModeBar with data-attribute-driven active state styling
/// - Toolbar with Bold/Italic/Link using MarkdownHandle
/// - Heading index sidebar
/// - Horizontal split layout
#[component]
fn App() -> Element {
    let mut current_mode = use_signal(|| Mode::LivePreview);

    rsx! {
        // Link your compiled Tailwind CSS here:
        // document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }

        div {
            class: "min-h-screen bg-gray-50 p-8",
            h1 {
                class: "text-2xl font-bold text-gray-900 mb-6",
                "Tailwind v4 Styled Markdown Editor"
            }
            markdown::Root {
                mode: current_mode,
                on_mode_change: move |m: Mode| current_mode.set(m),
                default_value: INITIAL_CONTENT,
                layout: Layout::Horizontal,
                class: "flex flex-col h-[600px] border border-gray-200 rounded-lg overflow-hidden bg-white shadow-sm",

                // ── Mode bar ──────────────────────────────────────────
                markdown::ModeBar {
                    class: "flex border-b border-gray-200 bg-gray-50",
                    markdown::ModeTab {
                        mode: Mode::Source,
                        class: "px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 \
                                data-[aria-selected=true]:text-blue-600 data-[aria-selected=true]:border-b-2 \
                                data-[aria-selected=true]:border-blue-600 focus:outline-none",
                        "Source"
                    }
                    markdown::ModeTab {
                        mode: Mode::LivePreview,
                        class: "px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 \
                                data-[aria-selected=true]:text-blue-600 data-[aria-selected=true]:border-b-2 \
                                data-[aria-selected=true]:border-blue-600 focus:outline-none",
                        "Live Preview"
                    }
                    markdown::ModeTab {
                        mode: Mode::Read,
                        class: "px-4 py-2 text-sm font-medium text-gray-600 hover:text-gray-900 \
                                data-[aria-selected=true]:text-blue-600 data-[aria-selected=true]:border-b-2 \
                                data-[aria-selected=true]:border-blue-600 focus:outline-none",
                        "Read"
                    }
                }

                // ── Toolbar ───────────────────────────────────────────
                markdown::Toolbar {
                    class: "flex items-center gap-0.5 border-b border-gray-200 bg-gray-50 px-2 py-1",
                    TailwindFormatToolbar {}
                }

                // ── Split pane: Editor + Divider + Preview ────────────
                div {
                    class: "flex flex-1 overflow-hidden",
                    markdown::Editor {
                        class: "flex-1 p-4 font-mono text-sm text-gray-800 \
                                bg-white resize-none focus:outline-none",
                        placeholder: "Type markdown here...",
                    }
                    markdown::Divider {
                        class: "w-px bg-gray-200 cursor-col-resize hover:bg-blue-400 transition-colors",
                    }
                    markdown::Preview {
                        class: "flex-1 p-4 overflow-auto prose prose-sm max-w-none",
                    }
                }

                // ── Sidebar: heading index ────────────────────────────
                TailwindHeadingSidebar {}
            }
        }
    }
}

/// Toolbar buttons wired to MarkdownHandle for Bold/Italic/Link.
/// Styled with Tailwind utility classes.
#[component]
fn TailwindFormatToolbar() -> Element {
    let handle: MarkdownHandle = use_markdown_handle();

    rsx! {
        markdown::ToolbarButton {
            class: "px-2 py-1 text-xs font-medium text-gray-700 rounded \
                    hover:bg-gray-200 active:bg-gray-300",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("**", "**").await;
                });
            },
            "Bold"
        }
        markdown::ToolbarSeparator {
            class: "w-px h-4 bg-gray-300 mx-1",
        }
        markdown::ToolbarButton {
            class: "px-2 py-1 text-xs font-medium text-gray-700 rounded \
                    hover:bg-gray-200 active:bg-gray-300",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("_", "_").await;
                });
            },
            "Italic"
        }
        markdown::ToolbarSeparator {
            class: "w-px h-4 bg-gray-300 mx-1",
        }
        markdown::ToolbarButton {
            class: "px-2 py-1 text-xs font-medium text-gray-700 rounded \
                    hover:bg-gray-200 active:bg-gray-300",
            onclick: move |_| {
                spawn(async move {
                    handle.wrap_selection("[", "](url)").await;
                });
            },
            "Link"
        }
    }
}

/// Sidebar showing the heading index (table of contents) with Tailwind classes.
#[component]
fn TailwindHeadingSidebar() -> Element {
    let headings = use_heading_index();

    rsx! {
        aside {
            class: "w-56 shrink-0 overflow-y-auto bg-gray-50 border-l border-gray-200 p-4 text-sm",
            h3 {
                class: "text-xs font-semibold text-gray-500 uppercase tracking-wider mb-3",
                "Contents"
            }
            ul {
                class: "space-y-1",
                for h in headings() {
                    li {
                        class: "text-gray-600 hover:text-gray-900 cursor-pointer truncate pl-2",
                        "H{h.level} {h.text}"
                    }
                }
            }
        }
    }
}
