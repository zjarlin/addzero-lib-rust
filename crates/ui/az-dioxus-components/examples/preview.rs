use std::fs;
use std::io;
use std::path::PathBuf;

use adui_dioxus::{ColumnAlign, Table, TableColumn, ThemeProvider};
use az_derive_aliases::{apply, plain_copy};
use dioxus::prelude::*;
use serde_json::{Value, json};

#[apply(plain_copy)]
struct RuntimeRow {
    node: &'static str,
    runtime: &'static str,
    region: &'static str,
    cpu: u32,
    latency_ms: u32,
    status: &'static str,
}

const RUNTIME_ROWS: [RuntimeRow; 6] = [
    RuntimeRow {
        node: "edge-01",
        runtime: "tokio-edge",
        region: "ap-shanghai",
        cpu: 42,
        latency_ms: 19,
        status: "healthy",
    },
    RuntimeRow {
        node: "edge-02",
        runtime: "tokio-edge",
        region: "ap-singapore",
        cpu: 57,
        latency_ms: 24,
        status: "healthy",
    },
    RuntimeRow {
        node: "edge-03",
        runtime: "tokio-batch",
        region: "eu-frankfurt",
        cpu: 76,
        latency_ms: 41,
        status: "warming",
    },
    RuntimeRow {
        node: "edge-04",
        runtime: "tokio-batch",
        region: "us-west",
        cpu: 33,
        latency_ms: 28,
        status: "healthy",
    },
    RuntimeRow {
        node: "edge-05",
        runtime: "ntex-stream",
        region: "cn-hangzhou",
        cpu: 88,
        latency_ms: 64,
        status: "throttled",
    },
    RuntimeRow {
        node: "edge-06",
        runtime: "ntex-stream",
        region: "eu-amsterdam",
        cpu: 49,
        latency_ms: 22,
        status: "healthy",
    },
];

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
        .join("az-dioxus-components-preview")
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
  <title>adui table preview</title>
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
  background: #f3f5f7;
  color: #171a21;
  font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
}

body {
  padding: 32px;
}

.table-preview {
  width: min(1120px, 100%);
  margin: 0 auto;
}

.table-preview .adui-theme-scope {
  width: 100%;
}

.table-preview .adui-table {
  border: 1px solid #d7dde6;
  background: #ffffff;
  box-shadow: 0 10px 30px rgba(17, 24, 39, 0.06);
}

.table-preview .adui-table-header {
  background: #f7f9fc;
  border-bottom: 1px solid #d7dde6;
}

.table-preview .adui-table-row:nth-child(even) {
  background: #fafbfd;
}

.table-preview .adui-table-row:hover {
  background: #f5f8fc;
}

.table-preview .adui-table-cell {
  padding: 14px 16px;
  font-size: 14px;
  border-bottom: 1px solid #e9edf3;
}

.table-preview .adui-table-cell-header {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: #5f6775;
}

.table-preview .adui-table-body-inner .adui-table-row:last-child .adui-table-cell {
  border-bottom: 0;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 92px;
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.status-pill--healthy {
  background: #e7f6ec;
  color: #1b6b41;
}

.status-pill--warming {
  background: #fff1db;
  color: #8a5b14;
}

.status-pill--throttled {
  background: #fde8e4;
  color: #a63b2d;
}
"#
}

#[allow(non_snake_case)]
#[component]
fn PreviewApp() -> Element {
    let columns = vec![
        TableColumn::new("node", "Node"),
        TableColumn::new("runtime", "Runtime"),
        TableColumn::new("region", "Region"),
        TableColumn::new("cpu", "CPU").align(ColumnAlign::Right),
        TableColumn::new("latency_ms", "Latency").align(ColumnAlign::Right),
        TableColumn::new("status", "Status")
            .align(ColumnAlign::Center)
            .render(render_status_cell),
    ];

    let data = RUNTIME_ROWS
        .into_iter()
        .map(|row| {
            json!({
                "node": row.node,
                "runtime": row.runtime,
                "region": row.region,
                "cpu": format!("{}%", row.cpu),
                "latency_ms": format!("{} ms", row.latency_ms),
                "status": row.status,
            })
        })
        .collect::<Vec<_>>();

    rsx! {
        ThemeProvider {
            div { class: "table-preview",
                Table {
                    columns,
                    data,
                    bordered: true,
                }
            }
        }
    }
}

fn render_status_cell(value: Option<&Value>, _record: &Value, _index: usize) -> Element {
    let status = value.and_then(Value::as_str).unwrap_or("unknown");
    let status_class = match status {
        "healthy" => "status-pill status-pill--healthy",
        "warming" => "status-pill status-pill--warming",
        "throttled" => "status-pill status-pill--throttled",
        _ => "status-pill",
    };

    rsx! {
        span { class: status_class, "{status}" }
    }
}
