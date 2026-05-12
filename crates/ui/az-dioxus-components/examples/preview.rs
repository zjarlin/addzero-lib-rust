use std::fs;
use std::io;
use std::path::PathBuf;

use az_dioxus_components::prelude::*;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct NodeRow {
    name: &'static str,
    zone: &'static str,
    load: &'static str,
    latency: &'static str,
    status: &'static str,
}

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
  <title>az-dioxus-components preview</title>
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
:root {
  --bg: #f4efe5;
  --panel: rgba(255, 250, 242, 0.92);
  --panel-strong: #fffaf0;
  --line: rgba(34, 32, 28, 0.14);
  --line-strong: rgba(34, 32, 28, 0.3);
  --ink: #1e1a16;
  --muted: #655c51;
  --accent: #b6542d;
  --accent-soft: rgba(182, 84, 45, 0.12);
  --ok: #2d6a4f;
  --warn: #8a5a00;
  --shadow: 0 24px 60px rgba(37, 28, 18, 0.12);
}

* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  min-height: 100%;
  color: var(--ink);
  background:
    radial-gradient(circle at top left, rgba(182, 84, 45, 0.18), transparent 34%),
    radial-gradient(circle at top right, rgba(79, 109, 122, 0.14), transparent 28%),
    linear-gradient(180deg, #f8f2e8 0%, #f1eadf 100%);
  font-family: "Avenir Next", "Segoe UI", sans-serif;
}

body::before {
  content: "";
  position: fixed;
  inset: 0;
  pointer-events: none;
  background-image:
    linear-gradient(rgba(24, 20, 16, 0.04) 1px, transparent 1px),
    linear-gradient(90deg, rgba(24, 20, 16, 0.04) 1px, transparent 1px);
  background-size: 24px 24px;
  mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.9), rgba(0, 0, 0, 0.25));
}

.preview-shell {
  max-width: 1240px;
  margin: 0 auto;
  padding: 40px 24px 72px;
  position: relative;
}

.preview-hero {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(300px, 0.75fr);
  gap: 24px;
  align-items: stretch;
  margin-bottom: 24px;
}

.preview-hero__intro,
.preview-hero__aside,
.az-card {
  background: var(--panel);
  backdrop-filter: blur(14px);
  border: 1px solid rgba(255, 255, 255, 0.65);
  box-shadow: var(--shadow);
}

.preview-hero__intro {
  border-radius: 28px;
  padding: 32px;
  position: relative;
  overflow: hidden;
}

.preview-hero__intro::after {
  content: "";
  position: absolute;
  top: 18px;
  right: 18px;
  width: 120px;
  height: 120px;
  border-radius: 24px;
  background: linear-gradient(135deg, rgba(182, 84, 45, 0.2), rgba(255, 255, 255, 0.1));
  border: 1px solid rgba(182, 84, 45, 0.28);
  transform: rotate(14deg);
}

.preview-kicker {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--line-strong);
  background: rgba(255, 255, 255, 0.46);
  font-size: 12px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.preview-title {
  margin: 18px 0 14px;
  max-width: 10ch;
  font-family: "Iowan Old Style", "Palatino Linotype", serif;
  font-size: clamp(48px, 8vw, 88px);
  line-height: 0.92;
  letter-spacing: -0.05em;
}

.preview-copy {
  max-width: 52ch;
  font-size: 16px;
  line-height: 1.7;
  color: var(--muted);
}

.preview-metrics {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
  margin-top: 22px;
}

.preview-metric {
  border-radius: 20px;
  padding: 14px 16px;
  background: rgba(255, 255, 255, 0.78);
  border: 1px solid var(--line);
}

.preview-metric__label {
  display: block;
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
}

.preview-metric__value {
  display: block;
  margin-top: 8px;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.04em;
}

.preview-hero__aside {
  border-radius: 28px;
  padding: 24px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.preview-aside__eyebrow,
.preview-card__eyebrow {
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--muted);
}

.preview-aside__status {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
}

.preview-aside__status::before {
  content: "";
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--ok);
  box-shadow: 0 0 0 8px rgba(45, 106, 79, 0.12);
}

.preview-aside__value {
  font-size: 58px;
  line-height: 1;
  letter-spacing: -0.06em;
  font-weight: 700;
}

.preview-aside__footnote {
  color: var(--muted);
  line-height: 1.6;
}

.preview-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(320px, 0.6fr);
  gap: 24px;
}

.az-card {
  border-radius: 28px;
}

.az-card__body {
  padding: 24px;
}

.preview-card__header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: end;
  margin-bottom: 18px;
}

.preview-card__title {
  margin: 6px 0 0;
  font-family: "Iowan Old Style", "Palatino Linotype", serif;
  font-size: 30px;
  line-height: 1.05;
}

.preview-card__hint {
  max-width: 30ch;
  color: var(--muted);
  font-size: 14px;
  line-height: 1.55;
  text-align: right;
}

.az-table__scroller {
  overflow-x: auto;
  border-radius: 22px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.72);
}

.az-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 720px;
}

.az-table__caption {
  caption-side: top;
  text-align: left;
  padding: 18px 18px 10px;
  font-size: 13px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}

.az-table__head {
  background: rgba(39, 31, 24, 0.04);
}

.az-table__header-cell,
.az-table__cell {
  padding: 16px 18px;
  border-bottom: 1px solid var(--line);
  vertical-align: middle;
}

.az-table__header-cell {
  font-size: 12px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
  font-weight: 700;
}

.az-table__row--selected {
  background: linear-gradient(90deg, rgba(182, 84, 45, 0.1), rgba(182, 84, 45, 0.04));
}

.az-table--striped .az-table__body .az-table__row:nth-child(even) {
  background: rgba(39, 31, 24, 0.025);
}

.az-table--bordered .az-table__header-cell + .az-table__header-cell,
.az-table--bordered .az-table__cell + .az-table__cell {
  border-left: 1px solid var(--line);
}

.az-table--dense .az-table__header-cell,
.az-table--dense .az-table__cell {
  padding-top: 13px;
  padding-bottom: 13px;
}

.az-table__cell--numeric {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.preview-node {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.preview-node__name {
  font-weight: 700;
}

.preview-node__meta {
  font-size: 13px;
  color: var(--muted);
}

.preview-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.86);
  font-size: 13px;
  white-space: nowrap;
}

.preview-chip::before {
  content: "";
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.preview-chip--ok {
  color: var(--ok);
}

.preview-chip--warn {
  color: var(--warn);
}

.preview-side-list {
  display: grid;
  gap: 12px;
}

.preview-side-item {
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.76);
}

.preview-side-item strong {
  display: block;
  font-size: 14px;
  margin-bottom: 6px;
}

.preview-side-item span {
  color: var(--muted);
  font-size: 14px;
  line-height: 1.55;
}

@media (max-width: 980px) {
  .preview-hero,
  .preview-grid {
    grid-template-columns: 1fr;
  }

  .preview-card__header {
    align-items: start;
    flex-direction: column;
  }

  .preview-card__hint {
    text-align: left;
  }
}

@media (max-width: 640px) {
  .preview-shell {
    padding: 18px 14px 48px;
  }

  .preview-hero__intro,
  .preview-hero__aside,
  .az-card__body {
    padding: 18px;
  }

  .preview-metrics {
    grid-template-columns: 1fr;
  }
}
"#
}

#[allow(non_snake_case)]
fn PreviewApp() -> Element {
    const NODES: [NodeRow; 4] = [
        NodeRow {
            name: "edge-gz-01",
            zone: "South cluster / Guangzhou",
            load: "71%",
            latency: "18ms",
            status: "stable",
        },
        NodeRow {
            name: "edge-sh-03",
            zone: "East cluster / Shanghai",
            load: "84%",
            latency: "26ms",
            status: "warming",
        },
        NodeRow {
            name: "edge-sz-02",
            zone: "South cluster / Shenzhen",
            load: "63%",
            latency: "14ms",
            status: "stable",
        },
        NodeRow {
            name: "edge-bj-05",
            zone: "North cluster / Beijing",
            load: "57%",
            latency: "31ms",
            status: "stable",
        },
    ];

    rsx! {
        div { class: "preview-shell",
            section { class: "preview-hero",
                div { class: "preview-hero__intro",
                    div { class: "preview-kicker", "az dioxus ui / field console" }
                    h1 { class: "preview-title", "Az Table" }
                    p { class: "preview-copy",
                        "A first-pass visual direction for the component library: editorial, operational, and slightly industrial. The goal is not a naked demo table, but a reusable shell where az-card and az-table already look like a real product surface."
                    }
                    div { class: "preview-metrics",
                        div { class: "preview-metric",
                            span { class: "preview-metric__label", "Nodes online" }
                            span { class: "preview-metric__value", "24" }
                        }
                        div { class: "preview-metric",
                            span { class: "preview-metric__label", "Median latency" }
                            span { class: "preview-metric__value", "19ms" }
                        }
                        div { class: "preview-metric",
                            span { class: "preview-metric__label", "Recovery window" }
                            span { class: "preview-metric__value", "04m" }
                        }
                    }
                }

                aside { class: "preview-hero__aside",
                    div {
                        div { class: "preview-aside__eyebrow", "Realtime posture" }
                        p { class: "preview-title", style: "font-size: 34px; max-width: none; margin-top: 10px;", "Control plane remains within guardrail." }
                    }
                    div {
                        div { class: "preview-aside__status", "healthy replication" }
                        div { class: "preview-aside__value", "99.94%" }
                        p { class: "preview-aside__footnote", "Table density, row state, and numeric alignment are already visible here, so later component refinement has a concrete GUI target instead of abstract props only." }
                    }
                }
            }

            section { class: "preview-grid",
                AzCard {
                    div { class: "preview-card__header",
                        div {
                            div { class: "preview-card__eyebrow", "Primary data surface" }
                            h2 { class: "preview-card__title", "Regional edge runtime roster" }
                        }
                        p { class: "preview-card__hint",
                            "The main table demonstrates striped rows, borders, selected state, and numeric alignment under an actual product-like shell."
                        }
                    }

                    AzTable {
                        class: "preview-ops-table",
                        striped: true,
                        bordered: true,
                        AzTableCaption { "Live region dispatch board" }
                        AzTableHead {
                            AzTableRow {
                                AzTableHeaderCell { "Node" }
                                AzTableHeaderCell { "Status" }
                                AzTableHeaderCell { numeric: true, "Load" }
                                AzTableHeaderCell { numeric: true, "Latency" }
                            }
                        }
                        AzTableBody {
                            {
                                NODES.iter().enumerate().map(|(index, node)| {
                                    let status_class = if node.status == "stable" {
                                        "preview-chip preview-chip--ok"
                                    } else {
                                        "preview-chip preview-chip--warn"
                                    };

                                    rsx! {
                                        AzTableRow { selected: index == 1,
                                            AzTableCell {
                                                div { class: "preview-node",
                                                    strong { class: "preview-node__name", "{node.name}" }
                                                    span { class: "preview-node__meta", "{node.zone}" }
                                                }
                                            }
                                            AzTableCell {
                                                span { class: status_class, "{node.status}" }
                                            }
                                            AzTableCell { numeric: true, "{node.load}" }
                                            AzTableCell { numeric: true, "{node.latency}" }
                                        }
                                    }
                                })
                            }
                        }
                    }
                }

                AzCard {
                    div { class: "preview-card__header",
                        div {
                            div { class: "preview-card__eyebrow", "Secondary notes" }
                            h2 { class: "preview-card__title", "What this proves" }
                        }
                    }
                    div { class: "preview-side-list",
                        div { class: "preview-side-item",
                            strong { "Single-crate preview loop" }
                            span { "You can render and inspect the GUI without wiring a browser runtime or app shell first." }
                        }
                        div { class: "preview-side-item",
                            strong { "CSS hook contract" }
                            span { "The `az-card` and `az-table` class names are already stable enough to grow a theme layer around them." }
                        }
                        div { class: "preview-side-item",
                            strong { "Component-first evolution" }
                            span { "Next additions can stay incremental: card header/footer, empty state, loading row, sortable header, and tokenized theme exports." }
                        }
                    }
                }
            }
        }
    }
}
