use std::fs;
use std::io;
use std::path::PathBuf;

use adui_dioxus::{ColumnAlign, Table, TableColumn, ThemeProvider};
use dioxus::prelude::*;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
struct AssetRow {
    name: &'static str,
    meta: &'static str,
    kind: &'static str,
    source: &'static str,
    version: &'static str,
    status: &'static str,
}

const ASSET_ROWS: [AssetRow; 6] = [
    AssetRow {
        name: "aio-plugin-runtime-notes.md",
        meta: "插件运行时说明与约束",
        kind: "Markdown",
        source: "workspace",
        version: "v5",
        status: "Indexed",
    },
    AssetRow {
        name: "demo-plugin.aio-plugin",
        meta: "插件包草案，待挂市场元数据",
        kind: "Plugin",
        source: "plugins",
        version: "v2",
        status: "Draft",
    },
    AssetRow {
        name: "aio-admin-prototype-board.html",
        meta: "当前后台工作台原型板",
        kind: "Prototype",
        source: "design",
        version: "v8",
        status: "Indexed",
    },
    AssetRow {
        name: "knowledge-sync-plan.md",
        meta: "知识同步切分与例外处理",
        kind: "Markdown",
        source: "docs",
        version: "v11",
        status: "Review",
    },
    AssetRow {
        name: "skill.sh",
        meta: "agent 侧稳定 CLI 入口脚本",
        kind: "Shell",
        source: "agents",
        version: "v4",
        status: "Active",
    },
    AssetRow {
        name: "drive-queue.json",
        meta: "Drive 同步排队快照",
        kind: "JSON",
        source: "runtime",
        version: "v1",
        status: "Queued",
    },
];

fn main() -> io::Result<()> {
    let html = build_front_document();
    let output_path = output_path()?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&output_path, html)?;
    println!("AIO front preview written to {}", output_path.display());
    Ok(())
}

fn output_path() -> io::Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join("target")
        .join("aio-front")
        .join("index.html"))
}

fn build_front_document() -> String {
    let body = dioxus_ssr::render_element(rsx!(AioFrontApp {}));

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>AIO Front</title>
  <style>{}</style>
</head>
<body>{}</body>
</html>"#,
        styles(),
        body
    )
}

fn styles() -> &'static str {
    r#"
* {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  min-height: 100%;
  background:
    radial-gradient(circle at top left, rgba(56, 100, 168, 0.08), transparent 28rem),
    linear-gradient(180deg, #f3f5f7 0%, #eef2f6 100%);
  color: #171a21;
  font-family: "Avenir Next", "IBM Plex Sans", "Segoe UI", sans-serif;
}

body {
  padding: 24px;
}

.aio-front {
  width: min(1480px, 100%);
  margin: 0 auto;
}

.aio-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 14px;
  border: 1px solid #d7dde6;
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.05);
}

.aio-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 168px;
}

.aio-brand__mark {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  background: #171a21;
  color: #ffffff;
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 0.08em;
}

.aio-brand__copy strong {
  display: block;
  font-size: 14px;
}

.aio-brand__copy span {
  display: block;
  margin-top: 2px;
  color: #667085;
  font-size: 12px;
}

.aio-axis,
.aio-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.aio-pill,
.aio-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 32px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid #d7dde6;
  background: #ffffff;
  color: #2f3441;
  font-size: 12px;
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.aio-pill--active {
  background: #171a21;
  border-color: #171a21;
  color: #ffffff;
}

.aio-shell {
  display: grid;
  grid-template-columns: 268px minmax(0, 1fr) 316px;
  gap: 16px;
  margin-top: 16px;
  min-height: calc(100vh - 104px);
}

.aio-rail,
.aio-main,
.aio-context {
  min-height: 0;
  border: 1px solid #d7dde6;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.04);
}

.aio-rail,
.aio-context {
  padding: 14px;
}

.aio-main {
  padding: 16px;
}

.aio-rail__title,
.aio-context__title {
  margin: 0;
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 0.04em;
}

.aio-rail__hint,
.aio-context__hint,
.aio-main__hint {
  margin: 6px 0 0;
  color: #667085;
  font-size: 12px;
  line-height: 1.55;
}

.tree-section + .tree-section,
.context-section + .context-section {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #e9edf3;
}

.tree-section__title {
  margin: 0 0 10px;
  color: #667085;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.tree-list {
  display: grid;
  gap: 6px;
}

.tree-item {
  display: block;
  padding: 10px 12px;
  border-radius: 12px;
  color: #2f3441;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
}

.tree-item--active {
  background: #f4f7fb;
  border: 1px solid #d7dde6;
}

.tree-children {
  display: grid;
  gap: 4px;
  margin-top: 6px;
  padding-left: 12px;
}

.tree-child {
  color: #667085;
  font-size: 12px;
}

.aio-main__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.aio-main__header h1 {
  margin: 0;
  font-size: 24px;
  line-height: 1.1;
  letter-spacing: -0.03em;
}

.metric-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin: 16px 0;
}

.metric {
  padding: 12px 14px;
  border: 1px solid #e9edf3;
  background: #f8fafc;
}

.metric strong {
  display: block;
  font-size: 22px;
  line-height: 1;
  letter-spacing: -0.04em;
}

.metric span {
  display: block;
  margin-top: 6px;
  color: #667085;
  font-size: 12px;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  margin-bottom: 12px;
  border: 1px solid #e9edf3;
  background: #f8fafc;
}

.table-toolbar__title strong {
  display: block;
  font-size: 13px;
}

.table-toolbar__title span {
  display: block;
  margin-top: 4px;
  color: #667085;
  font-size: 12px;
}

.table-toolbar__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 30px;
  padding: 0 10px;
  border-radius: 999px;
  border: 1px solid #d7dde6;
  background: #ffffff;
  color: #2f3441;
  font-size: 12px;
  font-weight: 700;
}

.adui-theme-scope {
  width: 100%;
}

.aio-assets-table {
  border: 1px solid #d7dde6;
  background: #ffffff;
}

.aio-assets-table .adui-table-header {
  background: #f7f9fc;
  border-bottom: 1px solid #d7dde6;
}

.aio-assets-table .adui-table-row:nth-child(even) {
  background: #fafbfd;
}

.aio-assets-table .adui-table-row:hover {
  background: #f4f7fb;
}

.aio-assets-table .adui-table-cell {
  padding: 13px 14px;
  border-bottom: 1px solid #e9edf3;
  font-size: 13px;
}

.aio-assets-table .adui-table-cell-header {
  color: #667085;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.aio-assets-table .adui-table-body-inner .adui-table-row:last-child .adui-table-cell {
  border-bottom: 0;
}

.asset-name {
  display: grid;
  gap: 3px;
}

.asset-name strong {
  font-size: 13px;
  font-weight: 700;
}

.asset-name span {
  color: #667085;
  font-size: 12px;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 82px;
  padding: 5px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.status-pill--indexed,
.status-pill--active {
  background: #e7f6ec;
  color: #1b6b41;
}

.status-pill--draft,
.status-pill--queued {
  background: #fff1db;
  color: #8a5b14;
}

.status-pill--review {
  background: #e8f0fc;
  color: #2f638f;
}

.row-actions {
  display: inline-flex;
  gap: 6px;
  flex-wrap: wrap;
}

.row-actions button {
  min-height: 26px;
  padding: 0 9px;
  border: 1px solid #d7dde6;
  background: #ffffff;
  color: #2f3441;
  font-size: 11px;
  font-weight: 700;
  border-radius: 999px;
}

.context-card {
  padding: 12px 14px;
  border: 1px solid #e9edf3;
  background: #f8fafc;
}

.context-card strong {
  display: block;
  font-size: 13px;
}

.context-card p {
  margin: 6px 0 0;
  color: #667085;
  font-size: 12px;
  line-height: 1.55;
}

.key-value {
  display: grid;
  gap: 10px;
}

.key-value__row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
}

.key-value__row span:first-child {
  color: #667085;
}

.key-value__row span:last-child {
  text-align: right;
  font-weight: 700;
}

.queue-list {
  display: grid;
  gap: 8px;
}

.queue-item {
  padding: 10px 12px;
  border: 1px solid #e9edf3;
  background: #ffffff;
}

.queue-item strong {
  display: block;
  font-size: 12px;
}

.queue-item span {
  display: block;
  margin-top: 4px;
  color: #667085;
  font-size: 12px;
}

@media (max-width: 1380px) {
  .aio-shell {
    grid-template-columns: 240px minmax(0, 1fr);
  }

  .aio-context {
    grid-column: 1 / -1;
  }
}
"#
}

#[allow(non_snake_case)]
#[component]
fn AioFrontApp() -> Element {
    let columns = vec![
        TableColumn::new("name", "名称").render(render_name_cell),
        TableColumn::new("kind", "类型"),
        TableColumn::new("source", "来源"),
        TableColumn::new("version", "版本").align(ColumnAlign::Right),
        TableColumn::new("status", "状态")
            .align(ColumnAlign::Center)
            .render(render_status_cell),
        TableColumn::new("actions", "操作")
            .align(ColumnAlign::Center)
            .render(render_actions_cell),
    ];

    let data = ASSET_ROWS
        .into_iter()
        .map(|row| {
            json!({
                "name": row.name,
                "meta": row.meta,
                "kind": row.kind,
                "source": row.source,
                "version": row.version,
                "status": row.status,
                "actions": "",
            })
        })
        .collect::<Vec<_>>();

    rsx! {
        ThemeProvider {
            div { class: "aio-front",
                div { class: "aio-topbar",
                    div { class: "aio-brand",
                        div { class: "aio-brand__mark", "AIO" }
                        div { class: "aio-brand__copy",
                            strong { "AIO Admin" }
                            span { "adui workbench prototype" }
                        }
                    }
                    div { class: "aio-axis",
                        span { class: "aio-pill", "工作台" }
                        span { class: "aio-pill aio-pill--active", "资产" }
                        span { class: "aio-pill", "运行" }
                        span { class: "aio-pill", "插件" }
                        span { class: "aio-pill", "系统" }
                    }
                    div { class: "aio-actions",
                        span { class: "aio-action", "同步队列 11" }
                        span { class: "aio-action", "知识镜像 97" }
                        span { class: "aio-action", "CLI 元数据" }
                    }
                }

                div { class: "aio-shell",
                    aside { class: "aio-rail",
                        h2 { class: "aio-rail__title", "侧轴上下文树" }
                        p { class: "aio-rail__hint", "当前主轴停在“资产”，左侧只展示资产域的模块子树。" }

                        section { class: "tree-section",
                            h3 { class: "tree-section__title", "个人资产" }
                            div { class: "tree-list",
                                span { class: "tree-item tree-item--active", "资产文件" }
                                span { class: "tree-item", "笔记" }
                                span { class: "tree-item", "安装包" }
                                span { class: "tree-item", "dotfiles" }
                            }
                        }

                        section { class: "tree-section",
                            h3 { class: "tree-section__title", "Agent 资产" }
                            div { class: "tree-list",
                                div { class: "tree-item",
                                    "Agent 资产总览"
                                    div { class: "tree-children",
                                        span { class: "tree-child", "Skills" }
                                        span { class: "tree-child", "CLI" }
                                        span { class: "tree-child", "MCP" }
                                    }
                                }
                            }
                        }
                    }

                    main { class: "aio-main",
                        div { class: "aio-main__header",
                            div {
                                h1 { "资产文件" }
                                p { class: "aio-main__hint", "首屏直接给对象、状态、主操作区和上下文细节，不做欢迎页。" }
                            }
                            div { class: "aio-actions",
                                span { class: "aio-action", "导入" }
                                span { class: "aio-action", "刷新索引" }
                                span { class: "aio-action", "生成分享" }
                            }
                        }

                        div { class: "metric-strip",
                            div { class: "metric",
                                strong { "128" }
                                span { "资产总数" }
                            }
                            div { class: "metric",
                                strong { "97" }
                                span { "已索引" }
                            }
                            div { class: "metric",
                                strong { "11" }
                                span { "待处理队列" }
                            }
                        }

                        div { class: "table-toolbar",
                            div { class: "table-toolbar__title",
                                strong { "资产目录" }
                                span { "adui Table 承载 filter / sort / row actions 的主工作面。" }
                            }
                            div { class: "table-toolbar__actions",
                                span { class: "toolbar-chip", "来源: 全部" }
                                span { class: "toolbar-chip", "状态: 已索引" }
                                span { class: "toolbar-chip", "排序: 最近更新" }
                            }
                        }

                        Table {
                            class: Some("aio-assets-table".to_string()),
                            columns,
                            data,
                            bordered: true,
                        }
                    }

                    aside { class: "aio-context",
                        h2 { class: "aio-context__title", "上下文面板" }
                        p { class: "aio-context__hint", "内容区聚焦对象，右侧面板承接解释、属性和最近动作。" }

                        section { class: "context-section",
                            div { class: "context-card",
                                strong { "当前对象" }
                                p { "aio-admin-prototype-board.html" }
                            }
                            div { class: "key-value", style: "margin-top: 12px;",
                                div { class: "key-value__row",
                                    span { "来源" }
                                    span { "design" }
                                }
                                div { class: "key-value__row",
                                    span { "版本" }
                                    span { "v8" }
                                }
                                div { class: "key-value__row",
                                    span { "状态" }
                                    span { "Indexed" }
                                }
                                div { class: "key-value__row",
                                    span { "说明" }
                                    span { "当前后台工作台原型板" }
                                }
                            }
                        }

                        section { class: "context-section",
                            h3 { class: "tree-section__title", "同步队列" }
                            div { class: "queue-list",
                                div { class: "queue-item",
                                    strong { "knowledge-sync-plan.md" }
                                    span { "等待异常归类后入 PG 镜像" }
                                }
                                div { class: "queue-item",
                                    strong { "drive-queue.json" }
                                    span { "待推送到远端 Git Pool" }
                                }
                                div { class: "queue-item",
                                    strong { "demo-plugin.aio-plugin" }
                                    span { "缺 market manifest，仍停留在草案态" }
                                }
                            }
                        }

                        section { class: "context-section",
                            h3 { class: "tree-section__title", "最近动作" }
                            div { class: "queue-list",
                                div { class: "queue-item",
                                    strong { "09:41" }
                                    span { "Skills 索引完成，落入 assets/agents/skills" }
                                }
                                div { class: "queue-item",
                                    strong { "09:12" }
                                    span { "Drive 队列写入 3 个待同步对象" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_name_cell(value: Option<&Value>, record: &Value, _index: usize) -> Element {
    let name = value.and_then(Value::as_str).unwrap_or_default();
    let meta = record.get("meta").and_then(Value::as_str).unwrap_or_default();

    rsx! {
        div { class: "asset-name",
            strong { "{name}" }
            span { "{meta}" }
        }
    }
}

fn render_status_cell(value: Option<&Value>, _record: &Value, _index: usize) -> Element {
    let status = value.and_then(Value::as_str).unwrap_or("Unknown");
    let class_name = match status {
        "Indexed" => "status-pill status-pill--indexed",
        "Active" => "status-pill status-pill--active",
        "Draft" => "status-pill status-pill--draft",
        "Queued" => "status-pill status-pill--queued",
        "Review" => "status-pill status-pill--review",
        _ => "status-pill",
    };

    rsx! {
        span { class: class_name, "{status}" }
    }
}

fn render_actions_cell(_value: Option<&Value>, record: &Value, _index: usize) -> Element {
    let asset_name = record.get("name").and_then(Value::as_str).unwrap_or_default();

    rsx! {
        div { class: "row-actions",
            button { r#type: "button", "查看" }
            button { r#type: "button", "打开" }
            button { r#type: "button", title: "{asset_name}", "选择" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aio_front_renders_domain_axis_and_adui_table() {
        let html = dioxus_ssr::render_element(rsx!(AioFrontApp {}));

        assert!(html.contains("AIO Admin"));
        assert!(html.contains("资产文件"));
        assert!(html.contains("adui-table"));
        assert!(html.contains("Agent 资产"));
    }

    #[test]
    fn aio_front_renders_status_pills_and_context_panel() {
        let html = dioxus_ssr::render_element(rsx!(AioFrontApp {}));

        assert!(html.contains("status-pill status-pill--indexed"));
        assert!(html.contains("上下文面板"));
        assert!(html.contains("knowledge-sync-plan.md"));
    }
}
