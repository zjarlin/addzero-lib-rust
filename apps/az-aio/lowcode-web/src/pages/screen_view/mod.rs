mod renderers;
use renderers::{TableScreen, MasterDetailScreen, AccordionScreen, FormScreen};

use crate::api;
use crate::model::{AppScreen, MetaFieldView};
use dioxus::prelude::*;

#[component]
pub fn ScreenView(
    screen_id: String,
    on_back: EventHandler<()>,
) -> Element {
    let mut screen = use_signal(|| None::<AppScreen>);
    let mut fields = use_signal(Vec::<MetaFieldView>::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        let sid = screen_id.clone();
        spawn(async move {
            if let Ok(Some(s)) = api::get_screen(&sid).await {
                let mid = s.model_id.clone();
                if let Ok(fs) = api::list_fields(&mid).await { fields.set(fs); }
                screen.set(Some(s));
            }
            loading.set(false);
        });
    });

    if loading() {
        return rsx! { div { style: "padding:40px; text-align:center; color:var(--adui-color-text-secondary);", "加载中..." } };
    }

    let s = screen.read();
    let s = match s.as_ref() {
        None => return rsx! {
            div { style: "display:flex; flex-direction:column; align-items:center; justify-content:center; height:100vh; gap:12px; background:var(--adui-color-bg-container);",
                p { style: "color:var(--adui-color-text-secondary); font-size:14px;", "未找到页面" }
                button {
                    style: "border:1px solid var(--adui-color-border); border-radius:6px; padding:6px 20px; cursor:pointer; background:transparent; color:var(--adui-color-text); font-size:13px;",
                    onclick: move |_| on_back.call(()),
                    "← 返回模型编辑"
                }
            }
        },
        Some(s) => s,
    };

    let layout = s.layout.as_str();
    let title = s.label.clone();
    let model_id = s.model_id.clone();
    let fields = fields.read().clone();
    let on_back_clone = on_back.clone();

    match layout {
        "Table" => rsx! { TableScreen { title, model_id, fields, on_back: move |_| on_back_clone.call(()) } },
        "MasterDetail" => rsx! { MasterDetailScreen { title, model_id, fields, on_back: move |_| on_back_clone.call(()) } },
        "Accordion" => rsx! { AccordionScreen { title, model_id, fields, on_back: move |_| on_back_clone.call(()) } },
        "Form" => rsx! { FormScreen { title, model_id, fields, on_back: move |_| on_back_clone.call(()) } },
        "TreeTable" | _ => rsx! { TableScreen { title, model_id, fields, on_back: move |_| on_back_clone.call(()) } },
    }
}
