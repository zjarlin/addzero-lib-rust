// Alternative styling: see examples/tailwind_styled.rs for Tailwind v4 CSS-first approach.

use dioxus::prelude::*;
use dioxus_nox_markdown::markdown;
use dioxus_nox_markdown::prelude::Mode;

fn main() {
    dioxus::launch(App);
}

/// Basic editor example: Source mode with a toolbar stub and word-count display.
///
/// Demonstrates:
/// - Uncontrolled default value via `default_value`
/// - `on_value_change` callback for derived state (word count)
/// - `markdown::Toolbar` with `ToolbarButton` and `ToolbarSeparator`
/// - `markdown::Editor` with placeholder
#[component]
fn App() -> Element {
    let mut word_count = use_signal(|| 0usize);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/examples.css") }
        markdown::Root {
            initial_mode: Mode::Source,
            default_value: "# Hello\n\nStart writing some **markdown** here.",
            on_value_change: move |v: String| {
                word_count.set(v.split_whitespace().count());
            },

            markdown::Toolbar {
                markdown::ToolbarButton { "Bold" }
                markdown::ToolbarSeparator {}
                markdown::ToolbarButton { "Italic" }
                markdown::ToolbarSeparator {}
                markdown::ToolbarButton { "Link" }
            }

            markdown::Editor {
                placeholder: "Type some markdown...",
            }

            p { "Word count: {word_count}" }
        }
    }
}
