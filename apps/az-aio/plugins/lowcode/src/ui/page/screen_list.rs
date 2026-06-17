use az_dioxus_components::{
    accordion::Accordion,
    status_badge::{StatusBadge, StatusBadgeTone},
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{FormRow, Input, Select, SelectOption},
    table::{Table, TableBody, TableCell, TableHead, TableHeaderCell, TableRow},
    workbench::{PageHeader, TableViewport, WorkbenchPage},
};
use dioxus::prelude::*;

use crate::backend::model::{AppScreenSummary, MetaModelSummary};
use crate::backend::record::RecordStore;
use crate::ui::page::helpers::{get_store, LowcodeActionForm};
use crate::ui::page::strategy::available_layouts;

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
    let layouts = available_layouts();

    // Pre-compute counts to avoid "let" inside rsx! (SSR limitation)
    let screens_with_counts: Vec<ScreenWithCount> = screens
        .iter()
        .map(|s| ScreenWithCount {
            count: rec_store.list(&s.model_id).len(),
            s: s.clone(),
        })
        .collect();

    rsx! {
        WorkbenchPage {
            PageHeader {
                title: "页面列表",
                subtitle: "低代码生成的页面 · 选择以预览",
                ToolbarButtonLink {
                    href: format!("/?route={lowcode_route}"),
                    "← 返回建模"
                }
            }
            Accordion { title: "＋ 新建页面",
                    LowcodeActionForm { action_name: "new-screen",
                        FormRow { label: "名称 (英文)", required: true,
                            Input { name: "scr_name", placeholder: "my-table", required: true }
                        }
                        FormRow { label: "标签", required: true,
                            Input { name: "scr_label", placeholder: "我的表格", required: true }
                        }
                        FormRow { label: "布局",
                            Select { name: "scr_layout", options: layout_options(&layouts, None) }
                        }
                        FormRow { label: "绑定模型",
                            Select { name: "scr_model_id", options: model_options(&models, None) }
                        }
                        ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "创建" }
                    }
            }
            TableViewport {
                Table { bordered: true, dense: true,
                    TableHead {
                        TableRow {
                            TableHeaderCell { "名称" }
                            TableHeaderCell { "标签" }
                            TableHeaderCell { "布局" }
                            TableHeaderCell { "绑定模型" }
                            TableHeaderCell { "记录数" }
                            TableHeaderCell { "操作" }
                        }
                    }
                    TableBody {
                        if screens_with_counts.is_empty() {
                            TableRow {
                                TableCell { class: "table-view__cell--empty", colspan: 6,
                                    "暂无页面"
                                }
                            }
                        } else {
                            for sc in &screens_with_counts {
                                TableRow {
                                    TableCell { code { "{sc.s.name}" } }
                                    TableCell { "{sc.s.label}" }
                                    TableCell {
                                        StatusBadge { "{layout_label_from_descriptors(&layouts, &sc.s.layout)}" }
                                    }
                                    TableCell { "{sc.s.model_name}" }
                                    TableCell {
                                        StatusBadge { tone: StatusBadgeTone::Accent, "{sc.count}" }
                                    }
                                    TableCell {
                                        ToolbarButtonLink {
                                            href: format!("/?route={lowcode_route}&screen={}", sc.s.id),
                                            "预览"
                                        }
                                        Accordion { title: "编辑", summary_class: "compact-summary",
                                                LowcodeActionForm {
                                                    action_name: "edit-screen",
                                                    hidden_fields: vec![("scr_id".to_string(), sc.s.id.clone())],
                                                    FormRow { label: "标签",
                                                        Input { name: "scr_label", value: sc.s.label.clone(), class: "form-input--compact" }
                                                    }
                                                    FormRow { label: "布局",
                                                        Select { name: "scr_layout", options: layout_options(&layouts, Some(sc.s.layout.clone())) }
                                                    }
                                                    FormRow { label: "绑定模型",
                                                        Select { name: "scr_model_id", options: model_options(&models, Some(sc.s.model_id.clone())) }
                                                    }
                                                    ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "保存" }
                                                }
                                        }
                                        ToolbarButtonLink {
                                            href: format!("/?route={lowcode_route}&mode=screens&action=delete-screen&scr_id={}", sc.s.id),
                                            tone: ToolbarButtonTone::Danger,
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

fn layout_label_from_descriptors(
    layouts: &[crate::contract::LowcodeLayoutDescriptor],
    code: &str,
) -> String {
    layouts
        .iter()
        .find(|layout| layout.code == code)
        .map(|layout| layout.label.clone())
        .unwrap_or_else(|| code.to_string())
}

fn layout_options(
    layouts: &[crate::contract::LowcodeLayoutDescriptor],
    current: Option<String>,
) -> Vec<SelectOption> {
    layouts
        .iter()
        .map(|layout| {
            SelectOption::new(layout.code.clone(), layout.label.clone())
                .selected(current.as_deref() == Some(layout.code.as_str()))
        })
        .collect()
}

fn model_options(models: &[MetaModelSummary], current: Option<String>) -> Vec<SelectOption> {
    models
        .iter()
        .map(|model| {
            SelectOption::new(model.id.clone(), format!("{} ({})", model.label, model.name))
                .selected(current.as_deref() == Some(model.id.as_str()))
        })
        .collect()
}
