use crate::api;
use crate::model::MetaFieldView;
use dioxus::prelude::*;
use std::collections::HashMap;

#[component]
pub fn MasterDetailScreen(
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

    let parent_key = fields.iter().find(|f| f.relation_type.as_deref() == Some("SelfRecursive"))
        .map(|f| f.name.clone()).unwrap_or_else(|| "parent_id".into());
    let label_key = fields.iter()
        .find(|f| f.field_type == "String" && f.name != "parent_id")
        .map(|f| f.name.clone()).unwrap_or_else(|| "name".into());

    if loading() {
        return rsx! { div { style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "加载中..." } };
    }

    let rlist = records.read();
    let mut children_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();
    for (i, r) in rlist.iter().enumerate() {
        let pid = r.get(&parent_key).and_then(|v| v.as_str()).unwrap_or("");
        if pid.is_empty() { roots.push(i); }
        else { children_map.entry(pid.to_string()).or_default().push(i); }
    }

    let detail_records: Vec<&serde_json::Value> = {
        let ex = expanded.read().clone();
        if ex.is_empty() { rlist.iter().collect() }
        else {
            let mut ids = vec![ex.clone()];
            let mut stack = vec![ex];
            while let Some(exp_id) = stack.pop() {
                if let Some(kids) = children_map.get(&exp_id) {
                    for &ki in kids {
                        let kid_id = rlist[ki].get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        ids.push(kid_id.clone());
                        stack.push(kid_id);
                    }
                }
            }
            rlist.iter().filter(|r| {
                let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
                ids.contains(&id.to_string())
            }).collect()
        }
    };

    let btn = "border:1px solid var(--adui-color-border); border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);";
    let danger = "border:1px solid #e5534b; border-radius:4px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:#e5534b;";
    let primary = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:4px 14px; font-size:12px; cursor:pointer; background:var(--adui-color-primary); color:#fff;";
    let modal_bg = "position:fixed; inset:0; background:rgba(0,0,0,0.45); display:flex; align-items:center; justify-content:center; z-index:1000;";
    let modal_box = "background:var(--adui-color-bg-container); border-radius:12px; padding:24px; min-width:480px; max-width:90vw; max-height:80vh; overflow:auto; box-shadow:0 8px 40px rgba(0,0,0,0.2);";
    let na = "background:var(--adui-color-primary); color:#fff; padding:4px 10px; border-radius:4px; font-size:12px; cursor:pointer; border:none;";
    let ni = "background:transparent; color:var(--adui-color-text); padding:4px 10px; border-radius:4px; font-size:12px; cursor:pointer; border:none;";

    // Build tree view data
    let tree_nodes: Vec<TreeViewNode> = roots.iter().map(|&idx| {
        let r = &rlist[idx];
        let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let label = r.get(&label_key).and_then(|v| v.as_str()).unwrap_or("?").to_string();
        TreeViewNode { id, label, depth: 0, children: build_children(idx, &rlist, &children_map, &label_key, 1) }
    }).collect();

    rsx! {
        div { style: "display:grid; grid-template-columns:260px 1fr; height:100vh; overflow:hidden; background:var(--adui-color-bg-layout);",
            aside { style: "display:flex; flex-direction:column; padding:12px; border-right:1px solid var(--adui-color-border); background:var(--adui-color-bg-container); overflow:auto;",
                button { style: "{btn}; margin-bottom:8px;", onclick: move |_| on_back.call(()), "← 返回" }
                strong { style: "font-size:13px; margin-bottom:6px; color:var(--adui-color-text);", "{title}" }
                button {
                    style: if expanded.read().is_empty() { "{na}; text-align:left; margin-bottom:4px;" } else { "{ni}; text-align:left; margin-bottom:4px;" },
                    onclick: move |_| expanded.set(String::new()),
                    "📂 全部 ({rlist.len()})"
                }
                {render_tree_nodes_flat(&tree_nodes, expanded)}
            }
            section { style: "display:flex; flex-direction:column; overflow:hidden; background:var(--adui-color-bg-container);",
                header { style: "display:flex; align-items:center; justify-content:space-between; padding:10px 20px; border-bottom:1px solid var(--adui-color-border);",
                    div {
                        h2 { style: "margin:0; font-size:16px;", "{title} · {detail_records.len()} 条" }
                        p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "左树右表布局" }
                    }
                    button { style: "{primary}", onclick: move |_| { create_vals.set(HashMap::new()); show_create.set(true); }, "+ 新建" }
                }
                div { style: "flex:1; overflow:auto;",
                    table { style: "width:100%; border-collapse:collapse; font-size:13px;",
                        thead { style: "position:sticky; top:0; z-index:2; background:var(--adui-color-bg-container);",
                            tr {
                                for f in &display_fields {
                                    th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary);", "{f.label}" }
                                }
                                th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary); width:80px;", "操作" }
                            }
                        }
                        tbody {
                            if detail_records.is_empty() {
                                tr { td { colspan: "{display_fields.len() + 1}", style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary); font-size:13px;", "暂无记录" } }
                            } else {
                                for r in &detail_records {
                                    tr { style: "border-bottom:1px solid var(--adui-color-border);",
                                        for f in &display_fields {
                                            td { style: "padding:6px 12px;", { r.get(&f.name).and_then(|v| v.as_str()).unwrap_or("") } }
                                        }
                                        td { style: "padding:6px 12px;",
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
                                    r#type: "text", style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text);",
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
    }
}

struct TreeViewNode {
    id: String,
    label: String,
    depth: usize,
    children: Vec<TreeViewNode>,
}

fn build_children(
    parent_idx: usize, rlist: &[serde_json::Value],
    children_map: &HashMap<String, Vec<usize>>, label_key: &str, depth: usize,
) -> Vec<TreeViewNode> {
    let r = &rlist[parent_idx];
    let pid = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
    match children_map.get(pid) {
        None => vec![],
        Some(kids) => kids.iter().map(|&idx| {
            let kr = &rlist[idx];
            let id = kr.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let label = kr.get(label_key).and_then(|v| v.as_str()).unwrap_or("?").to_string();
            TreeViewNode {
                id, label, depth,
                children: build_children(idx, rlist, children_map, label_key, depth + 1),
            }
        }).collect(),
    }
}

fn render_tree_nodes_flat(nodes: &[TreeViewNode], mut expanded: Signal<String>) -> Element {
    let na = "background:var(--adui-color-primary); color:#fff; padding:4px 10px; border-radius:4px; font-size:12px; cursor:pointer; border:none;";
    let ni = "background:transparent; color:var(--adui-color-text); padding:4px 10px; border-radius:4px; font-size:12px; cursor:pointer; border:none;";
    rsx! {
        for node in nodes {
            {
                let pad = node.depth * 20 + 8;
                let nid = node.id.clone();
                let nlabel = node.label.clone();
                let has_kids = !node.children.is_empty();
                let is_active = *expanded.read() == nid;
                rsx! {
                    div { style: "padding-left:{pad}px;",
                        button {
                            style: if is_active { "{na}; margin:1px 0; width:100%; text-align:left; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;" } else { "{ni}; margin:1px 0; width:100%; text-align:left; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;" },
                            onclick: { let id2 = nid.clone(); move |_| expanded.set(id2.clone()) },
                            if has_kids { "📁 " } else { "📄 " } "{nlabel}"
                        }
                        {render_tree_nodes_flat(&node.children, expanded)}
                    }
                }
            }
        }
    }
}
