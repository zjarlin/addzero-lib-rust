use dioxus::prelude::*;

use crate::model::AppScreenSummary;
use crate::page::helpers::{get_store, layout_label};
use crate::record::RecordStore;

struct ScreenWithCount {
    s: AppScreenSummary,
    count: usize,
}

pub fn render_screen_list_page() -> Element {
    let store = get_store();
    let screens = store.list_screens_sync();
    let models = store.list_models_sync();
    let lowcode_route = "/lowcode";
    let rec_store = RecordStore::global();

    // Pre-compute counts to avoid "let" inside rsx! (SSR limitation)
    let screens_with_counts: Vec<ScreenWithCount> = screens
        .iter()
        .map(|s| ScreenWithCount {
            count: rec_store.list(&s.model_id).len(),
            s: s.clone(),
        })
        .collect();

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "页面列表" }
                p { "低代码生成的页面 · 选择以预览" }
                a {
                    href: "/?route={lowcode_route}",
                    class: "toolbar-button",
                    style: "margin-top: 8px;",
                    "← 返回建模"
                }
            }
            details { class: "lowcode-accordion",
                summary { class: "lowcode-accordion__summary", "＋ 新建页面" }
                div { class: "lowcode-accordion__body",
                    form {
                        method: "get",
                        action: "/",
                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                        input { r#type: "hidden", name: "action", value: "new-screen" }
                        div { class: "settings-form-row",
                            label { "名称 (英文)" }
                            input { class: "settings-input", name: "scr_name", placeholder: "my-table", required: "required" }
                        }
                        div { class: "settings-form-row",
                            label { "标签" }
                            input { class: "settings-input", name: "scr_label", placeholder: "我的表格", required: "required" }
                        }
                        div { class: "settings-form-row",
                            label { "布局" }
                            select { class: "settings-input", name: "scr_layout",
                                option { value: "Table", "增删改查表格" }
                                option { value: "MasterDetail", "左树右表" }
                                option { value: "Accordion", "手风琴" }
                                option { value: "Form", "表单" }
                                option { value: "TreeTable", "树形表格" }
                            }
                        }
                        div { class: "settings-form-row",
                            label { "绑定模型" }
                            select { class: "settings-input", name: "scr_model_id",
                                for m in &models {
                                    option { value: "{m.id}", "{m.label} ({m.name})" }
                                }
                            }
                        }
                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "创建" }
                    }
                }
            }
            div { class: "lowcode-table-scroll",
                table { class: "az-table az-table--bordered az-table--dense",
                    thead {
                        tr {
                            th { class: "az-table__header-cell", "名称" }
                            th { class: "az-table__header-cell", "标签" }
                            th { class: "az-table__header-cell", "布局" }
                            th { class: "az-table__header-cell", "绑定模型" }
                            th { class: "az-table__header-cell", "记录数" }
                            th { class: "az-table__header-cell", "操作" }
                        }
                    }
                    tbody { class: "az-table__body",
                        if screens_with_counts.is_empty() {
                            tr {
                                td { class: "az-table__cell az-table__cell--empty", colspan: "6",
                                    "暂无页面"
                                }
                            }
                        } else {
                            for sc in &screens_with_counts {
                                tr {
                                    td { class: "az-table__cell", code { "{sc.s.name}" } }
                                    td { class: "az-table__cell", "{sc.s.label}" }
                                    td { class: "az-table__cell",
                                        span { class: "az-badge", "{layout_label(&sc.s.layout)}" }
                                    }
                                    td { class: "az-table__cell", "{sc.s.model_name}" }
                                    td { class: "az-table__cell",
                                        span { class: "az-badge az-badge--accent", "{sc.count}" }
                                    }
                                    td { class: "az-table__cell",
                                        a {
                                            href: "/?route={lowcode_route}&screen={sc.s.id}",
                                            class: "toolbar-button",
                                            "预览"
                                        }
                                        details { class: "lowcode-accordion",
                                            summary { class: "lowcode-accordion__summary", style: "font-size: 10px; padding: 2px 6px;", "编辑" }
                                            div { class: "lowcode-accordion__body",
                                                form {
                                                    method: "get",
                                                    action: "/",
                                                    input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                    input { r#type: "hidden", name: "action", value: "edit-screen" }
                                                    input { r#type: "hidden", name: "scr_id", value: "{sc.s.id}" }
                                                    div { class: "settings-form-row",
                                                        label { "标签" }
                                                        input { class: "settings-input", name: "scr_label", value: "{sc.s.label}", style: "font-size: 12px;" }
                                                    }
                                                    div { class: "settings-form-row",
                                                        label { "布局" }
                                                        select { class: "settings-input", name: "scr_layout",
                                                            option { value: "Table", selected: sc.s.layout == "Table", "增删改查表格" }
                                                            option { value: "MasterDetail", selected: sc.s.layout == "MasterDetail", "左树右表" }
                                                            option { value: "Accordion", selected: sc.s.layout == "Accordion", "手风琴" }
                                                            option { value: "Form", selected: sc.s.layout == "Form", "表单" }
                                                            option { value: "TreeTable", selected: sc.s.layout == "TreeTable", "树形表格" }
                                                        }
                                                    }
                                                    div { class: "settings-form-row",
                                                        label { "绑定模型" }
                                                        select { class: "settings-input", name: "scr_model_id",
                                                            for m in &models {
                                                                option { value: "{m.id}", selected: sc.s.model_id == m.id, "{m.label} ({m.name})" }
                                                            }
                                                        }
                                                    }
                                                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                                                }
                                            }
                                        }
                                        a {
                                            href: "/?route={lowcode_route}&mode=screens&action=delete-screen&scr_id={sc.s.id}",
                                            class: "toolbar-button toolbar-button--danger",
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
