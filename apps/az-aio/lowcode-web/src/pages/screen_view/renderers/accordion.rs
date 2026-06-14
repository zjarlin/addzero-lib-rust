use crate::api;
use crate::model::MetaFieldView;
use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn AccordionScreen(
    title: String,
    model_id: String,
    fields: Vec<MetaFieldView>,
    on_back: EventHandler<()>,
) -> Element {
    let mut records = use_signal(Vec::<serde_json::Value>::new);
    let mut loading = use_signal(|| true);
    let mut expanded = use_signal(String::new);
    let mut show_create = use_signal(|| false);
    let mut create_vals = use_signal(HashMap::<String, String>::new);

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

    let btn = "border:1px solid var(--adui-color-border); border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);";
    let danger = "border:1px solid #e5534b; border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:#e5534b;";
    let primary = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:4px 14px; font-size:12px; cursor:pointer; background:var(--adui-color-primary); color:#fff;";
    let modal_bg = "position:fixed; inset:0; background:rgba(0,0,0,0.45); display:flex; align-items:center; justify-content:center; z-index:1000;";
    let modal_box = "background:var(--adui-color-bg-container); border-radius:12px; padding:24px; min-width:480px; max-width:90vw; max-height:80vh; overflow:auto; box-shadow:0 8px 40px rgba(0,0,0,0.2);";

    if loading() {
        return rsx! { div { style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "加载中..." } };
    }

    let rec_list = records.read();
    let df_first = display_fields.first().map(|f| f.name.clone()).unwrap_or_else(|| "name".into());

    rsx! {
        div { style: "display:flex; flex-direction:column; height:100vh; overflow:hidden; background:var(--adui-color-bg-container);",
            header { style: "display:flex; align-items:center; justify-content:space-between; padding:10px 20px; border-bottom:1px solid var(--adui-color-border);",
                div { style: "display:flex; align-items:center; gap:12px;",
                    button { style: "{btn}", onclick: move |_| on_back.call(()), "← 返回" }
                    div {
                        h2 { style: "margin:0; font-size:17px;", "{title}" }
                        p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "手风琴 · {rec_list.len()} 条记录" }
                    }
                }
                button { style: "{primary}", onclick: move |_| { create_vals.set(HashMap::new()); show_create.set(true); }, "+ 新建" }
            }
            div { style: "flex:1; overflow:auto; padding:16px;",
                if rec_list.is_empty() {
                    p { style: "text-align:center; color:var(--adui-color-text-secondary); padding:40px;", "暂无记录" }
                } else {
                    for rec in rec_list.iter() {
                        {
                            let rid = rec.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let is_open = *expanded.read() == rid;
                            let label = rec.get(&df_first).and_then(|v| v.as_str()).unwrap_or("?");
                            rsx! {
                                div { style: "border:1px solid var(--adui-color-border); border-radius:8px; margin-bottom:8px; overflow:hidden;",
                                    div {
                                        style: "display:flex; align-items:center; justify-content:space-between; padding:10px 16px; cursor:pointer; background:var(--adui-color-bg-layout);",
                                        onclick: {
                                            let rid2 = rid.clone();
                                            move |_| {
                                                let cur = expanded.read().clone();
                                                expanded.set(if cur == rid2 { String::new() } else { rid2.clone() });
                                            }
                                        },
                                        strong { style: "font-size:13px; color:var(--adui-color-text);", "{label}" }
                                        span { style: "font-size:16px; color:var(--adui-color-text-secondary);", if is_open { "▾" } else { "▸" } }
                                    }
                                    if is_open {
                                        div { style: "padding:12px 16px;",
                                            for f in display_fields.iter() {
                                                div { style: "display:flex; padding:4px 0; border-bottom:1px solid var(--adui-color-split);",
                                                    span { style: "width:120px; font-size:12px; color:var(--adui-color-text-secondary);", "{f.label}" }
                                                    span { style: "flex:1; font-size:13px; color:var(--adui-color-text);",
                                                        { rec.get(&f.name).and_then(|v| v.as_str()).unwrap_or("") }
                                                    }
                                                }
                                            }
                                            div { style: "display:flex; justify-content:flex-end; gap:8px; margin-top:10px;",
                                                {
                                                    let drid = rid.clone();
                                                    let dmid = model_id.clone();
                                                    rsx! {
                                                        button {
                                                            style: "{danger}",
                                                            onclick: {
                                                                let rc = drid.clone();
                                                                let mc = dmid.clone();
                                                                move |_| {
                                                                    let rc2 = rc.clone();
                                                                    let mc2 = mc.clone();
                                                                    spawn(async move {
                                                                        if api::delete_record(&mc2, &rc2).await.is_ok() {
                                                                            if let Ok(rs) = api::list_records(&mc2).await { records.set(rs); }
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
            }
        }
        if show_create() {
            div { style: "{modal_bg}", onclick: move |_| show_create.set(false),
                div { style: "{modal_box}", onclick: move |ev: MouseEvent| ev.stop_propagation(),
                    h3 { style: "margin:0 0 16px; font-size:16px; color:var(--adui-color-text);", "新建记录" }
                    div { style: "display:flex; flex-direction:column; gap:10px; max-height:50vh; overflow:auto;",
                        for f in &fields {
                            div { style: "display:flex; flex-direction:column; gap:3px;",
                                label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "{f.label} ({f.name})" }
                                input { r#type: "text", style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text);",
                                    oninput: { let fname = f.name.clone(); move |ev: FormEvent| { let mut m = create_vals(); m.insert(fname.clone(), ev.value()); create_vals.set(m); } },
                                }
                            }
                        }
                    }
                    div { style: "display:flex; justify-content:flex-end; gap:8px; margin-top:20px;",
                        button { style: "{btn}", onclick: move |_| show_create.set(false), "取消" }
                        button { style: "{primary}",
                            onclick: {
                                let cmid = model_id.clone();
                                move |_| {
                                    let vals = create_vals.read().clone(); let cmid2 = cmid.clone();
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
    }
}
