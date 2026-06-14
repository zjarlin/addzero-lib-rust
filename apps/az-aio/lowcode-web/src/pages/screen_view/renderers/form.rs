use crate::api;
use crate::model::MetaFieldView;
use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn FormScreen(
    title: String,
    model_id: String,
    fields: Vec<MetaFieldView>,
    on_back: EventHandler<()>,
) -> Element {
    let mut records = use_signal(Vec::<serde_json::Value>::new);
    let mut loading = use_signal(|| true);
    let mut form_vals = use_signal(HashMap::<String, String>::new);
    let mut saved_count = use_signal(|| 0u32);

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

    let non_rel: Vec<MetaFieldView> = fields.iter()
        .filter(|f| f.field_type != "Relation")
        .cloned()
        .collect();

    let btn = "border:1px solid var(--adui-color-border); border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);";
    let danger = "border:1px solid #e5534b; border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:#e5534b;";
    let primary = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:6px 20px; font-size:13px; cursor:pointer; background:var(--adui-color-primary); color:#fff;";

    if loading() {
        return rsx! { div { style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "加载中..." } };
    }

    rsx! {
        div { style: "display:flex; flex-direction:column; height:100vh; overflow:hidden; background:var(--adui-color-bg-container);",
            header { style: "display:flex; align-items:center; gap:12px; padding:10px 20px; border-bottom:1px solid var(--adui-color-border);",
                button { style: "{btn}", onclick: move |_| on_back.call(()), "← 返回" }
                div {
                    h2 { style: "margin:0; font-size:17px;", "{title} · 表单" }
                    p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "已保存 {saved_count()} 条 · 共 {records.read().len()} 条" }
                }
            }
            div { style: "display:grid; grid-template-columns:400px 1fr; height:100%;",
                div { style: "display:flex; flex-direction:column; gap:12px; padding:20px; border-right:1px solid var(--adui-color-border); overflow:auto;",
                    strong { style: "font-size:14px; color:var(--adui-color-text);", "新建 {title}" }
                    for f in &fields {
                        div { style: "display:flex; flex-direction:column; gap:3px;",
                            label { style: "font-size:12px; color:var(--adui-color-text-secondary);",
                                "{f.label}" if f.is_required { span { style: "color:var(--adui-color-error);", " *" } } }
                            input { r#type: "text", style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text);",
                                oninput: { let fname = f.name.clone(); move |ev: FormEvent| { let mut m = form_vals(); m.insert(fname.clone(), ev.value()); form_vals.set(m); } },
                            }
                        }
                    }
                    button {
                        style: "{primary}",
                        onclick: {
                            let cmid = model_id.clone();
                            move |_| {
                                let vals = form_vals.read().clone(); let cmid2 = cmid.clone();
                                spawn(async move {
                                    if api::create_record(&cmid2, &vals).await.is_ok() {
                                        if let Ok(rs) = api::list_records(&cmid2).await { records.set(rs); }
                                        saved_count.set(saved_count() + 1);
                                        form_vals.set(HashMap::new());
                                    }
                                });
                            }
                        },
                        "提交"
                    }
                }
                div { style: "flex:1; overflow:auto; padding:20px;",
                    table { style: "width:100%; border-collapse:collapse; font-size:13px;",
                        thead {
                            tr {
                                for f in &non_rel {
                                    th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary);", "{f.label}" }
                                }
                                th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary); width:80px;", "操作" }
                            }
                        }
                        tbody {
                            if records.read().is_empty() {
                                tr { td { colspan: "{non_rel.len() + 1}", style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "暂无记录" } }
                            } else {
                                for r in records.read().iter() {
                                    tr { style: "border-bottom:1px solid var(--adui-color-border);",
                                        for f in &non_rel {
                                            td { style: "padding:6px 12px;", { r.get(&f.name).and_then(|v| v.as_str()).unwrap_or("") } }
                                        }
                                        td {
                                            {
                                                let drid = r.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                                let dmid = model_id.clone();
                                                rsx! {
                                                    button {
                                                        style: "{danger}",
                                                        onclick: {
                                                            let rid_c = drid.clone();
                                                            let mid_c = dmid.clone();
                                                            move |_| {
                                                                let rid_c2 = rid_c.clone();
                                                                let mid_c2 = mid_c.clone();
                                                                spawn(async move {
                                                                    if api::delete_record(&mid_c2, &rid_c2).await.is_ok() {
                                                                        if let Ok(rs) = api::list_records(&mid_c2).await { records.set(rs); }
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
}
