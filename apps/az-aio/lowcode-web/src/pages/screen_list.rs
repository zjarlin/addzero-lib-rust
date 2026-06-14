use crate::api;
use crate::model::{AppScreenSummary, CreateScreenInput, MetaModelSummary};
use adui_dioxus::components::button::{Button, ButtonSize};
use adui_dioxus::components::input::Input;
use dioxus::prelude::*;

const LAYOUTS: &[(&str, &str)] = &[
    ("Table", "表格"),
    ("MasterDetail", "左树右表"),
    ("TreeTable", "树形表"),
    ("Accordion", "手风琴"),
    ("Form", "表单"),
];

#[component]
pub fn ScreenList(
    models: ReadOnlySignal<Vec<MetaModelSummary>>,
    on_view_screen: EventHandler<String>,
) -> Element {
    let mut screens = use_signal(Vec::<AppScreenSummary>::new);
    let mut show_create = use_signal(|| false);
    let mut scr_name = use_signal(String::new);
    let mut scr_label = use_signal(String::new);
    let mut scr_layout = use_signal(|| "Table".to_string());
    let mut scr_model_id = use_signal(String::new);

    use_effect(move || { spawn(async move { if let Ok(l) = api::list_screens().await { screens.set(l); } }); });

    let _btn_style = "border:1px solid var(--adui-color-border); border-radius:6px; padding:3px 10px; font-size:11px; cursor:pointer; background:transparent; color:var(--adui-color-text);";
    let primary_btn = "border:1px solid var(--adui-color-primary); border-radius:6px; padding:4px 12px; font-size:12px; cursor:pointer; background:var(--adui-color-primary); color:#fff;";

    rsx! {
        div { style: "display:flex; flex-direction:column; gap:8px; padding:8px 0;",
            strong { style: "font-size:13px; color:var(--adui-color-text);", "AppScreen 页面" }
            p { style: "font-size:11px; color:var(--adui-color-text-secondary); margin:0;", "基于模型生成交互页面" }
            button {
                style: "{primary_btn}; align-self:flex-start;",
                onclick: move |_| show_create.set(!show_create()),
                if show_create() { "取消" } else { "+ 新建页面" }
            }
            if show_create() {
                div { style: "display:flex; flex-direction:column; gap:6px; padding:10px; border-radius:8px; background:var(--adui-color-bg-layout);",
                    Input { placeholder: Some("标识 (name)".into()), value: scr_name(), on_change: move |v| scr_name.set(v) }
                    Input { placeholder: Some("显示标签 (label)".into()), value: scr_label(), on_change: move |v| scr_label.set(v) }
                    label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "绑定模型" }
                    select { style: "padding:6px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); margin-bottom:4px;",
                        onchange: move |ev: FormEvent| scr_model_id.set(ev.value()),
                        option { value: "", "— 选择模型 —" }
                        for m in models.read().iter() {
                            option { value: "{m.id}", "{m.label} ({m.name})" }
                        }
                    }
                    label { style: "font-size:12px; color:var(--adui-color-text-secondary);", "布局类型" }
                    div { style: "display:flex; flex-wrap:wrap; gap:5px; margin-bottom:6px;",
                        { let cl = scr_layout.read().clone(); rsx! {
                            for (lv, ll) in LAYOUTS.iter() {
                                { let v=lv.to_string(); let s=cl==v; let st = if s { "border:1px solid var(--adui-color-primary); color:var(--adui-color-primary); padding:3px 10px; border-radius:6px; font-size:12px; cursor:pointer; background:transparent;" } else { "border:1px solid var(--adui-color-border); color:var(--adui-color-text); padding:3px 10px; border-radius:6px; font-size:12px; cursor:pointer; background:transparent;" }; rsx!{
                                    button { style: "{st}", onclick: { let v2=v.clone(); move |_| scr_layout.set(v2.clone()) }, "{ll}" }
                                }}
                            }
                        }}
                    }
                    Button { size: ButtonSize::Small, onclick: move |_| {
                        let name = scr_name.read().clone(); let label = scr_label.read().clone();
                        let layout = scr_layout.read().clone(); let model_id = scr_model_id.read().clone();
                        if name.is_empty() || model_id.is_empty() { return; }
                        spawn(async move {
                            let inp = CreateScreenInput { name, label: if label.is_empty(){"未命名".into()}else{label}, layout, model_id, config_json: "{}".into() };
                            if api::create_screen(&inp).await.is_ok() { if let Ok(l)=api::list_screens().await { screens.set(l); show_create.set(false); } }
                        });
                    }, "创建" }
                }
            }
            div { style: "flex:1; overflow:auto;",
                if screens.read().is_empty() {
                    p { style: "font-size:12px; color:var(--adui-color-text-secondary); padding:12px 0;", "暂无页面，请先创建" }
                } else {
                    for s in screens.read().iter() {
                        div { style: "display:flex; align-items:center; justify-content:space-between; padding:6px 10px; border-radius:6px; margin-bottom:2px; cursor:pointer;",
                            onclick: { let sid = s.id.clone(); move |_| on_view_screen.call(sid.clone()) },
                            div { style: "flex:1;",
                                strong { style: "font-size:13px; color:var(--adui-color-text);", "{s.label}" }
                                div { style: "font-size:11px; color:var(--adui-color-text-secondary);", "{s.model_name} · {s.layout}" }
                            }
                            button {
                                style: "border:1px solid #e5534b; border-radius:4px; padding:1px 8px; font-size:10px; background:transparent; color:#e5534b; cursor:pointer;",
                                onclick: { let sid = s.id.clone(); move |ev: MouseEvent| { ev.stop_propagation(); let sid2 = sid.clone(); spawn(async move { if api::delete_screen(&sid2).await.is_ok() { if let Ok(l)=api::list_screens().await { screens.set(l); } } }); } },
                                "删除"
                            }
                        }
                    }
                }
            }
        }
    }
}
