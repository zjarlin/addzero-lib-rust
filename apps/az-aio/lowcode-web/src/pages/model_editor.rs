use crate::api;
use crate::model::{CreateFieldInput, CreateModelInput, MetaFieldView, MetaModelSummary, UpdateFieldInput};
use crate::pages::{ScreenList};
use adui_dioxus::components::button::{Button, ButtonSize};
use adui_dioxus::components::input::Input;
use adui_dioxus::components::tree::Tree;
use adui_dioxus::components::select_base::TreeNode;
use adui_dioxus::components::modal::Modal;
use adui_dioxus::theme;
use dioxus::prelude::*;
use std::rc::Rc;

const TYPE_OPTS: &[(&str, &str)] = &[
    ("String","字符串"),("Integer","整数"),("Float","浮点数"),
    ("Boolean","布尔"),("DateTime","日期时间"),("Json","JSON"),("Relation","关联"),
];

const REL_OPTS: &[(&str, &str)] = &[
    ("","—"),("OneToOne","一对一"),("OneToMany","一对多"),("ManyToMany","多对多"),("SelfRecursive","自递归"),
];

#[component]
pub fn ModelEditor(on_view_screen: EventHandler<String>) -> Element {
    let theme = theme::use_theme();
    let mut models = use_signal(Vec::<MetaModelSummary>::new);
    let mut selected_model_id = use_signal(String::new);
    let mut fields = use_signal(Vec::<MetaFieldView>::new);
    let mut search = use_signal(String::new);

    let mut show_new_model = use_signal(|| false);
    let mut new_name = use_signal(String::new);
    let mut new_label = use_signal(String::new);
    let mut new_desc = use_signal(String::new);

    let mut field_modal = use_signal(|| false);
    let mut editing_field = use_signal(|| None::<MetaFieldView>);
    let mut f_name = use_signal(String::new);
    let mut f_label = use_signal(String::new);
    let mut f_type = use_signal(|| "String".to_string());
    let mut f_rel = use_signal(String::new);
    let mut f_rel_model = use_signal(String::new);
    let mut f_req = use_signal(|| false);
    let mut f_uniq = use_signal(|| false);

    let mut page = use_signal(|| 0u8);

    use_effect(move || { spawn(async move { if let Ok(l) = api::list_models().await { models.set(l); } }); });

    let trigger = selected_model_id();
    use_effect(move || {
        let mid = trigger.clone();
        if mid.is_empty() { return; }
        spawn(async move { if let Ok(l) = api::list_fields(&mid).await { fields.set(l); } });
    });

    let q = search.read().to_lowercase();
    let filtered: Vec<MetaModelSummary> = if q.is_empty() {
        models.read().clone()
    } else {
        models.read().iter().filter(|m| m.label.to_lowercase().contains(&q) || m.name.to_lowercase().contains(&q)).cloned().collect()
    };

    let tree_data: Vec<TreeNode> = filtered.iter().map(|m| TreeNode {
        key: m.id.clone(), label: m.label.clone(), disabled: false, children: vec![],
    }).collect();

    let filter_fn: Option<Rc<dyn Fn(&TreeNode) -> bool>> = {
        let sq = search.read().to_lowercase();
        if sq.is_empty() {
            None
        } else {
            Some(Rc::new(move |node: &TreeNode| {
                node.label.to_lowercase().contains(&sq) || node.key.to_lowercase().contains(&sq)
            }))
        }
    };

    let cur_sel = if selected_model_id.read().is_empty() { vec![] } else { vec![selected_model_id.read().clone()] };
    let sel_model = models.read().iter().find(|m| m.id == *selected_model_id.read()).cloned();

    let tab_base = "flex:1; padding:5px 0; border:none; border-radius:6px; font-size:12px; cursor:pointer;";
    let tab_active = "background:var(--adui-color-primary); color:#fff";
    let tab_inactive = "background:transparent; color:var(--adui-color-text)";
    let model_tab = format!("{tab_base} {}", if page()==0 {tab_active} else {tab_inactive});
    let screen_tab = format!("{tab_base} {}", if page()==1 {tab_active} else {tab_inactive});

    let btn_style = "border:1px solid var(--adui-color-border); border-radius:6px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);".to_string();
    let primary_btn = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:4px 14px; font-size:12px; cursor:pointer; background:var(--adui-color-primary); color:#fff;".to_string();
    let danger_btn = "border:1px solid #e5534b; border-radius:4px; padding:2px 8px; font-size:11px; background:transparent; color:#e5534b; cursor:pointer;".to_string();

    let is_dark = theme.theme().mode == theme::ThemeMode::Dark;
    let theme_switch_label = if is_dark { "☀ 日间模式" } else { "🌙 夜间模式" };

    rsx! {
        div { style: "display:grid; grid-template-columns:300px 1fr; height:100vh; overflow:hidden; background:var(--adui-color-bg-layout);",
            aside { style: "display:flex; flex-direction:column; padding:14px; gap:8px; border-right:1px solid var(--adui-color-border); background:var(--adui-color-bg-container); overflow:hidden;",
                div { style: "display:flex; gap:4px; margin-bottom:4px;",
                    button { style: "{model_tab}", onclick: move |_| page.set(0), "模型" }
                    button { style: "{screen_tab}", onclick: move |_| page.set(1), "页面" }
                }
                // 搜索框置顶
                Input { placeholder: Some("搜索模型...".into()), value: search(), allow_clear: true,
                    on_change: move |v: String| search.set(v),
                }
                // 新建模型按钮在搜索框下方
                button {
                    style: "{btn_style}; align-self:flex-start; margin-bottom:4px;",
                    onclick: { let mut s = show_new_model; move |_| s.set(!s()) },
                    if show_new_model() { "取消" } else { "+ 新建模型" }
                }
                if show_new_model() {
                    div { style: "display:flex; flex-direction:column; gap:6px; padding:10px; border-radius:8px; background:var(--adui-color-bg-layout);",
                        Input { placeholder: Some("英文标识".into()), value: new_name(), on_change: move |v| new_name.set(v) }
                        Input { placeholder: Some("中文标签".into()), value: new_label(), on_change: move |v| new_label.set(v) }
                        Input { placeholder: Some("描述".into()), value: new_desc(), on_change: move |v| new_desc.set(v) }
                        Button { size: ButtonSize::Small, onclick: move |_| {
                            let nm = new_name.read().clone(); if nm.is_empty() { return; }
                            let lb = new_label.read().clone(); let dc = new_desc.read().clone();
                            spawn(async move {
                                if api::create_model(&CreateModelInput{name:nm,label:if lb.is_empty(){"未命名".into()}else{lb},description:dc}).await.is_ok()
                                { if let Ok(l)=api::list_models().await { models.set(l); } show_new_model.set(false); }
                            });
                        }, "创建" }
                    }
                }
                if page() == 0 {
                    div { style: "flex:1; overflow:auto;",
                        Tree { tree_data: tree_data.clone(), selectable: true, block_node: true, show_icon: false,
                            selected_keys: cur_sel,
                            filter_tree_node: filter_fn,
                            on_select: move |keys: Vec<String>| { selected_model_id.set(keys.first().cloned().unwrap_or_default()); },
                        }
                    }
                } else {
                    ScreenList { models, on_view_screen: move |sid| on_view_screen.call(sid) }
                }
                // 主题切换开关
                div { style: "border-top:1px solid var(--adui-color-border); padding-top:8px; margin-top:auto;",
                    button {
                        style: "{btn_style}; width:100%;",
                        onclick: move |_| {
                            let h = theme;
                            let new_mode = if h.theme().mode == theme::ThemeMode::Dark { theme::ThemeMode::Light } else { theme::ThemeMode::Dark };
                            h.set_mode(new_mode);
                        },
                        "{theme_switch_label}"
                    }
                }
            }

            // RIGHT PANEL
            section { style: "display:flex; flex-direction:column; overflow:hidden; background:var(--adui-color-bg-container);",
                header { style: "display:flex; align-items:center; justify-content:space-between; padding:12px 20px; border-bottom:1px solid var(--adui-color-border);",
                    match &sel_model {
                        Some(m) => rsx! {
                            div {
                                h2 { style: "margin:0; font-size:17px;", "{m.label} · 字段" }
                                p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "{m.name} — {m.description}" }
                            }
                        },
                        None => rsx! { div { h2 { style: "margin:0; font-size:17px;", "字段定义" }
                            p { style: "margin:1px 0 0; font-size:11px; color:var(--adui-color-text-secondary);", "← 选择模型查看字段" } } },
                    }
                    if sel_model.is_some() && page() == 0 {
                        button {
                            style: "{primary_btn}",
                            onclick: move |_| { editing_field.set(None); f_name.set(String::new()); f_label.set(String::new()); f_type.set("String".into()); f_rel.set(String::new()); f_rel_model.set(String::new()); f_req.set(false); f_uniq.set(false); field_modal.set(true); },
                            "+ 添加字段"
                        }
                    } else if sel_model.is_some() && page() == 1 {
                        p { style: "font-size:12px; color:var(--adui-color-text-secondary);", "在左侧创建 AppScreen" }
                    }
                }
                div { style: "flex:1; overflow:auto;",
                    table { style: "width:100%; border-collapse:collapse; font-size:13px;",
                        thead { style: "position:sticky; top:0; z-index:2; background:var(--adui-color-bg-container);",
                            tr {
                                for h in &["字段名","标签","类型","必填","唯一","关联","操作"] {
                                    th { style: "text-align:left; padding:7px 12px; border-bottom:2px solid var(--adui-color-border); font-weight:600; font-size:12px; color:var(--adui-color-text-secondary);", "{h}" }
                                }
                            }
                        }
                        tbody {
                            if fields.read().is_empty() {
                                tr { td { colspan: "7", style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary); font-size:13px;", "暂无字段，请点击右上角「添加字段」" } }
                            } else {
                                for fv in fields.read().iter() {
                                    tr { style: "border-bottom:1px solid var(--adui-color-border);",
                                        td { style: "padding:6px 12px;", code { style: "font-size:11px; color:var(--adui-color-primary);", "{fv.name}" } }
                                        td { style: "padding:6px 12px;", "{fv.label}" }
                                        td { style: "padding:6px 12px;",
                                            span { style: "font-size:11px; padding:1px 8px; border-radius:10px; background:var(--adui-color-bg-layout);", "{ft_label(&fv.field_type)}" }
                                        }
                                        td { style: "padding:6px 12px;", if fv.is_required { "✓" } }
                                        td { style: "padding:6px 12px;", if fv.is_unique { "✓" } }
                                        td { style: "padding:6px 12px; font-size:11px;", "{rel_short(fv)}" }
                                        td { style: "padding:6px 12px; white-space:nowrap;",
                                            button {
                                                style: "border:1px solid var(--adui-color-border); border-radius:4px; padding:2px 8px; font-size:11px; background:transparent; color:var(--adui-color-text); cursor:pointer; margin-right:4px;",
                                                onclick: { let fv2 = fv.clone(); move |_| {
                                                    editing_field.set(Some(fv2.clone())); f_name.set(fv2.name.clone()); f_label.set(fv2.label.clone()); f_type.set(fv2.field_type.clone());
                                                    f_rel.set(fv2.relation_type.clone().unwrap_or_default()); f_rel_model.set(fv2.relation_model_id.clone().unwrap_or_default());
                                                    f_req.set(fv2.is_required); f_uniq.set(fv2.is_unique); field_modal.set(true);
                                                }},
                                                "编辑"
                                            }
                                            button {
                                                style: "{danger_btn}",
                                                onclick: { let fid = fv.id.clone(); let mid = selected_model_id.read().clone();
                                                    move |_| { let fid2 = fid.clone(); let mid2 = mid.clone(); spawn(async move {
                                                        if api::delete_field(&fid2).await.is_ok() { if let Ok(l)=api::list_fields(&mid2).await { fields.set(l); } }
                                                    });}
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

        // FIELD MODAL
        Modal {
            open: field_modal(),
            title: Some(if editing_field.read().is_some() { "编辑字段".into() } else { "新建字段".into() }),
            on_cancel: move |_| field_modal.set(false),
            on_ok: {
                let mid = selected_model_id.read().clone(); let is_edit = editing_field.read().is_some();
                let edit_id = editing_field.read().as_ref().map(|f| f.id.clone());
                move |_| {
                    let name = f_name.read().clone(); let label = f_label.read().clone(); let ft = f_type.read().clone();
                    let rel = f_rel.read().clone(); let rel_m = f_rel_model.read().clone();
                    if name.is_empty() || mid.is_empty() { return; }
                    let mid2 = mid.clone(); let eid = edit_id.clone();
                    spawn(async move {
                        if is_edit {
                            if let Some(ref fid) = eid {
                                let inp = UpdateFieldInput { name:Some(name), label:Some(if label.is_empty(){"未命名".into()}else{label}), field_type:Some(ft), is_required:Some(f_req()), is_unique:Some(f_uniq()), order:None, default_value:None,
                                    relation_type: if rel.is_empty() { None } else { Some(rel) }, relation_model_id: if rel_m.is_empty() { None } else { Some(rel_m) },
                                };
                                if api::update_field(fid, &inp).await.is_ok() { if let Ok(l)=api::list_fields(&mid2).await { fields.set(l); } }
                            }
                        } else {
                            let inp = CreateFieldInput { name, label: if label.is_empty(){"未命名".into()}else{label}, field_type:ft, is_required:f_req(), is_unique:f_uniq(), order:0, default_value:None,
                                relation_type: if rel.is_empty() { None } else { Some(rel) }, relation_model_id: if rel_m.is_empty() { None } else { Some(rel_m) },
                            };
                            if api::create_field(&mid2, &inp).await.is_ok() { if let Ok(l)=api::list_fields(&mid2).await { fields.set(l); } }
                        }
                    });
                    field_modal.set(false);
                }
            },
            div { style: "display:flex; flex-direction:column; gap:10px; padding:4px 0;",
                Input { placeholder: Some("字段名 (英文)".into()), value: f_name(), on_change: move |v| f_name.set(v) }
                Input { placeholder: Some("标签 (中文)".into()), value: f_label(), on_change: move |v| f_label.set(v) }
                div { style: "display:flex; flex-wrap:wrap; gap:5px;",
                    { let cft = f_type.read().clone(); rsx! {
                        for (val, label) in TYPE_OPTS.iter() {
                            { let v=val.to_string(); let s=cft==v; let st = if s { "border:1px solid var(--adui-color-primary); color:var(--adui-color-primary); padding:3px 10px; border-radius:6px; font-size:12px; cursor:pointer; background:transparent;" } else { "border:1px solid var(--adui-color-border); color:var(--adui-color-text); padding:3px 10px; border-radius:6px; font-size:12px; cursor:pointer; background:transparent;" }; rsx!{
                                button { style: "{st}", onclick: { let v2=v.clone(); move |_| f_type.set(v2.clone()) }, "{label}" }
                            }}
                        }
                    }}
                }
                if f_type() == "Relation" {
                    div { style: "display:flex; gap:8px;",
                        div { style: "flex:1;",
                            label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "关联类型" }
                            select { style: "width:100%; padding:6px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); margin-top:4px;",
                                onchange: move |ev: FormEvent| f_rel.set(ev.value()),
                                for (rv, rl) in REL_OPTS { option { value: "{rv}", selected: f_rel() == *rv, "{rl}" } }
                            }
                        }
                        div { style: "flex:1;",
                            label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "关联模型" }
                            select { style: "width:100%; padding:6px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); margin-top:4px;",
                                onchange: move |ev: FormEvent| f_rel_model.set(ev.value()),
                                option { value: "", "—" }
                                for m in models.read().iter() {
                                    option { value: "{m.id}", selected: f_rel_model() == m.id, "{m.label} ({m.name})" }
                                }
                            }
                        }
                    }
                }
                div { style: "display:flex; gap:16px; margin-top:4px;",
                    label { style: "display:flex; align-items:center; gap:4px; font-size:12px;",
                        input { r#type: "checkbox", checked: f_req(), onchange: move |_| f_req.set(!f_req()) }
                        "必填"
                    }
                    label { style: "display:flex; align-items:center; gap:4px; font-size:12px;",
                        input { r#type: "checkbox", checked: f_uniq(), onchange: move |_| f_uniq.set(!f_uniq()) }
                        "唯一"
                    }
                }
            }
        }
    }
}

fn ft_label(ft: &str) -> &str {
    match ft { "String"=>"字符串","Integer"=>"整数","Float"=>"浮点数","Boolean"=>"布尔","DateTime"=>"日期时间","Json"=>"JSON","Relation"=>"关联", _=>ft }
}

fn rel_short(f: &MetaFieldView) -> &str {
    if f.field_type != "Relation" { return "—"; }
    match f.relation_type.as_deref() { Some("OneToOne")=>"1:1", Some("OneToMany")=>"1:N", Some("ManyToMany")=>"N:N", Some("SelfRecursive")=>"树", _=>"" }
}
