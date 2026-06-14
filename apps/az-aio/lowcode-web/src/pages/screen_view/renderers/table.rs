use crate::api;
use crate::model::MetaFieldView;
use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn TableScreen(
    title: String,
    model_id: String,
    fields: Vec<MetaFieldView>,
    on_back: EventHandler<()>,
) -> Element {
    let mut records = use_signal(Vec::<serde_json::Value>::new);
    let mut loading = use_signal(|| true);
    let mut search = use_signal(String::new);
    let mut show_create = use_signal(|| false);
    let mut create_vals = use_signal(HashMap::<String, String>::new);
    let mut show_edit = use_signal(|| false);
    let mut editing_rid = use_signal(String::new);
    let mut edit_vals = use_signal(HashMap::<String, String>::new);

    {
        let m = model_id.clone();
        use_effect(move || {
            let m2 = m.clone();
            spawn(async move {
                if let Ok(rs) = api::list_records(&m2).await { records.set(rs); }
                loading.set(false);
            });
        });
    }

    let display_fields: Vec<MetaFieldView> = fields.iter()
        .filter(|f| f.field_type != "Relation")
        .cloned()
        .collect();

    if loading() {
        return rsx! { div { style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "加载中..." } };
    }

    let rlist = records.read();
    let sq = search.read().to_lowercase();
    let filtered: Vec<&serde_json::Value> = if sq.is_empty() {
        rlist.iter().collect()
    } else {
        rlist.iter().filter(|r| {
            r.as_object().map_or(false, |obj| {
                obj.values().any(|v| v.as_str().map_or(false, |s| s.to_lowercase().contains(&sq)))
            })
        }).collect()
    };

    let btn = "border:1px solid var(--adui-color-border); border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);";
    let danger = "border:1px solid #e5534b; border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:#e5534b;";
    let primary = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:4px 14px; font-size:12px; cursor:pointer; background:var(--adui-color-primary); color:#fff;";
    let modal_bg = "position:fixed; inset:0; background:rgba(0,0,0,0.45); display:flex; align-items:center; justify-content:center; z-index:1000;";
    let modal_box = "background:var(--adui-color-bg-container); border-radius:12px; padding:24px; min-width:480px; max-width:90vw; max-height:80vh; overflow:auto; box-shadow:0 8px 40px rgba(0,0,0,0.2);";

    rsx! {
        div { style: "display:flex; flex-direction:column; height:100vh; overflow:hidden; background:var(--adui-color-bg-container);",
            header { style: "display:flex; align-items:center; gap:12px; padding:10px 20px; border-bottom:1px solid var(--adui-color-border);",
                button { style: "{btn}", onclick: move |_| on_back.call(()), "← 返回" }
                div { style: "flex:1;",
                    h2 { style: "margin:0; font-size:17px;", "{title}" }
                    p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "表格 · {filtered.len()} 条记录" }
                }
                div { style: "display:flex; gap:8px; align-items:center;",
                    input {
                        r#type: "text", placeholder: "搜索...", value: search(),
                        style: "padding:5px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); width:200px;",
                        oninput: move |ev: FormEvent| search.set(ev.value()),
                    }
                    button {
                        style: "{primary}",
                        onclick: move |_| { create_vals.set(HashMap::new()); show_create.set(true); },
                        "+ 新建"
                    }
                }
            }
            div { style: "flex:1; overflow:auto;",
                table { style: "width:100%; border-collapse:collapse; font-size:13px;",
                    thead { style: "position:sticky; top:0; z-index:2; background:var(--adui-color-bg-container);",
                        tr {
                            for f in &display_fields {
                                th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary);", "{f.label}" }
                            }
                            th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary); width:140px;", "操作" }
                        }
                    }
                    tbody {
                        if filtered.is_empty() {
                            tr { td { colspan: "{display_fields.len() + 1}", style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary); font-size:13px;", "暂无记录" } }
                        } else {
                            for r in &filtered {
                                tr { style: "border-bottom:1px solid var(--adui-color-border);",
                                    for f in &display_fields {
                                        td { style: "padding:6px 12px;", { r.get(&f.name).and_then(|v| v.as_str()).unwrap_or("") } }
                                    }
                                    td { style: "padding:6px 12px; white-space:nowrap;",
                                        {
                                            let rid = r.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            let vals: HashMap<String, String> = r.as_object().unwrap().iter()
                                                .filter_map(|(k,v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect();
                                            let dmid = model_id.clone();
                                            let erid = rid.clone();
                                            let evals = vals.clone();
                                            rsx! {
                                                button {
                                                    style: "{btn}; margin-right:4px;",
                                                    onclick: move |_| {
                                                        editing_rid.set(erid.clone());
                                                        edit_vals.set(evals.clone());
                                                        show_edit.set(true);
                                                    },
                                                    "编辑"
                                                }
                                                button {
                                                    style: "{danger}",
                                                    onclick: {
                                                        let drid = rid.clone();
                                                        let dmid2 = dmid.clone();
                                                        move |_| {
                                                            let drid2 = drid.clone();
                                                            let dmid3 = dmid2.clone();
                                                            spawn(async move {
                                                                if api::delete_record(&dmid3, &drid2).await.is_ok() {
                                                                    if let Ok(rs) = api::list_records(&dmid3).await { records.set(rs); }
                                                                }
                                                            });
                                                        }
                                                    },
                                                    "删除"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // ── Create Modal ──
        if show_create() {
            div { style: "{modal_bg}", onclick: move |_| show_create.set(false),
                div { style: "{modal_box}", onclick: move |ev: MouseEvent| ev.stop_propagation(),
                    h3 { style: "margin:0 0 16px; font-size:16px; color:var(--adui-color-text);", "新建记录" }
                    div { style: "display:flex; flex-direction:column; gap:10px; max-height:50vh; overflow:auto;",
                        for f in &fields {
                            div { style: "display:flex; flex-direction:column; gap:3px;",
                                label { style: "font-size:12px; color:var(--adui-color-text-secondary);",
                                    "{f.label} ({f.name})" if f.is_required { span { style: "color:var(--adui-color-error);", " *" } } }
                                input {
                                    r#type: "text",
                                    style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text);",
                                    oninput: { let fname = f.name.clone(); move |ev: FormEvent| { let mut m = create_vals(); m.insert(fname.clone(), ev.value()); create_vals.set(m); } },
                                }
                            }
                        }
                    }
                    div { style: "display:flex; justify-content:flex-end; gap:8px; margin-top:20px;",
                        button { style: "{btn}", onclick: move |_| show_create.set(false), "取消" }
                        button {
                            style: "{primary}",
                            onclick: {
                                let cmid = model_id.clone();
                                move |_| {
                                    let vals = create_vals.read().clone();
                                    let cmid2 = cmid.clone();
                                    spawn(async move {
                                        if api::create_record(&cmid2, &vals).await.is_ok() {
                                            if let Ok(rs) = api::list_records(&cmid2).await { records.set(rs); }
                                        }
                                    });
                                    show_create.set(false);
                                }
                            },
                            "保存"
                        }
                    }
                }
            }
        }
        // ── Edit Modal ──
        if show_edit() {
            div { style: "{modal_bg}", onclick: move |_| show_edit.set(false),
                div { style: "{modal_box}", onclick: move |ev: MouseEvent| ev.stop_propagation(),
                    h3 { style: "margin:0 0 16px; font-size:16px; color:var(--adui-color-text);", "编辑记录" }
                    div { style: "display:flex; flex-direction:column; gap:10px; max-height:50vh; overflow:auto;",
                        for f in fields.iter().filter(|f| f.name != "id") {
                            div { style: "display:flex; flex-direction:column; gap:3px;",
                                label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "{f.label} ({f.name})" }
                                input {
                                    r#type: "text",
                                    style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text);",
                                    value: edit_vals.read().get(&f.name).cloned().unwrap_or_default(),
                                    oninput: { let fname = f.name.clone(); move |ev: FormEvent| { let mut m = edit_vals(); m.insert(fname.clone(), ev.value()); edit_vals.set(m); } },
                                }
                            }
                        }
                    }
                    div { style: "display:flex; justify-content:flex-end; gap:8px; margin-top:20px;",
                        button { style: "{btn}", onclick: move |_| show_edit.set(false), "取消" }
                        button {
                            style: "{primary}",
                            onclick: {
                                let emid = model_id.clone();
                                let erid2 = editing_rid.read().clone();
                                move |_| {
                                    let vals = edit_vals.read().clone();
                                    let emid2 = emid.clone();
                                    let erid3 = erid2.clone();
                                    spawn(async move {
                                        if api::update_record(&emid2, &erid3, &vals).await.is_ok() {
                                            if let Ok(rs) = api::list_records(&emid2).await { records.set(rs); }
                                        }
                                    });
                                    show_edit.set(false);
                                }
                            },
                            "保存"
                        }
                    }
                }
            }
        }
    }
}
