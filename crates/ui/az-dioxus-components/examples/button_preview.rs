use std::fs;
use std::io;
use std::path::PathBuf;

use adui_dioxus::{Button, ButtonProps, ButtonType, ThemeProvider};
use dioxus::prelude::*;

fn main() -> io::Result<()> {
    let preview_html = build_preview_document();
    let output_path = preview_output_path()?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, preview_html)?;
    println!("Preview written to {}", output_path.display());
    Ok(())
}

fn preview_output_path() -> io::Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join("target")
        .join("az-dioxus-components-button-preview")
        .join("index.html"))
}

fn build_preview_document() -> String {
    let body = dioxus_ssr::render_element(rsx!(PreviewApp {}));

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>az dioxus button preview</title>
  <style>{}</style>
</head>
<body>{}</body>
</html>"#,
        preview_styles(),
        body
    )
}

fn preview_styles() -> &'static str {
    r#"
* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  min-height: 100%;
  background: #f6f8fb;
  color: #101828;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
}

body {
  padding: 32px;
}

.preview {
  width: min(960px, 100%);
  margin: 0 auto;
  display: grid;
  gap: 20px;
}

.preview__row {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.demo-button {
  min-width: 160px;
}

.adui-btn.preview-button {
  box-shadow: inset 0 0 0 1px rgba(22, 119, 255, 0.18);
}
"#
}

#[allow(non_snake_case)]
#[component]
fn PreviewApp() -> Element {
    rsx! {
        ThemeProvider {
            div { class: "preview",
                div { class: "preview__row",
                    DemoButton {
                        r#type: ButtonType::Primary,
                        class: Some("preview-button".to_string()),
                        "Wrapped Button"
                    }
                    Button {
                        r#type: ButtonType::Default,
                        "Raw Button"
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn DemoButton(mut props: ButtonProps) -> Element {
    props.class = Some(match props.class.take() {
        Some(class) if !class.is_empty() => format!("demo-button {class}"),
        _ => "demo-button".to_string(),
    });

    rsx! {
        Button { ..props }
    }
}
