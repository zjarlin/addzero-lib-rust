// Alternative styling: see examples/tailwind_styled.rs for Tailwind v4 CSS-first approach.

use dioxus::prelude::*;
use dioxus_nox_markdown::markdown;
use dioxus_nox_markdown::prelude::{Mode, use_heading_index};

fn main() {
    dioxus::launch(App);
}

const SAMPLE: &str = "\
---
title: Read-Only Demo
author: nox-markdown
---

# Introduction

This is a **read-only** display example using `dioxus-nox-markdown`.

## Section One

Content in section one with some _italic_ text.

## Section Two

More content with a [link](https://example.com).

### Subsection

Deeper nesting works too.
";

/// Read-only example: controlled value in Read mode with a heading index sidebar.
///
/// Demonstrates:
/// - Controlled `value` prop with `Mode::Read`
/// - `use_heading_index()` hook for building a table of contents
/// - `markdown::Content` for rendering the parsed document
#[component]
fn App() -> Element {
    let value = use_signal(|| SAMPLE.to_string());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/examples.css") }
        markdown::Root {
            initial_mode: Mode::Read,
            value: value,
            DocView {}
        }
    }
}

/// Inner component that consumes `MarkdownContext` via hooks.
/// Must be a descendant of `markdown::Root`.
#[component]
fn DocView() -> Element {
    let headings = use_heading_index();

    rsx! {
        div {
            aside {
                h2 { "Table of Contents" }
                ul {
                    for h in headings() {
                        li { "H{h.level}: {h.text}" }
                    }
                }
            }
            markdown::Content {}
        }
    }
}
