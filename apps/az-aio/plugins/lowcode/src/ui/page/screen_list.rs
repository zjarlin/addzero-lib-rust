use az_dioxus_components::{
    az_accordion::AzAccordion,
    az_badge::{AzBadge, AzBadgeTone},
    az_button::{AzButton, AzButtonLink, AzButtonTone},
    az_form::{AzFormRow, AzInput, AzSelect, AzSelectOption},
    az_table::{AzTable, AzTableBody, AzTableCell, AzTableHead, AzTableHeaderCell, AzTableRow},
    az_workbench::{AzPageHeader, AzTableViewport, AzWorkbenchPage},
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
        AzWorkbenchPage {
            AzPageHeader {
                title: "页面列表",
                subtitle: "低代码生成的页面 · 选择以预览",
                AzButtonLink {
                    href: format!("/?route={lowcode_route}"),
                    "← 返回建模"
                }
            }
            AzAccordion { title: "＋ 新建页面",
                    LowcodeActionForm { action_name: "new-screen",
                        AzFormRow { label: "名称 (英文)", required: true,
                            AzInput { name: "scr_name", placeholder: "my-table", required: true }
                        }
                        AzFormRow { label: "标签", required: true,
                            AzInput { name: "scr_label", placeholder: "我的表格", required: true }
                        }
                        AzFormRow { label: "布局",
                            AzSelect { name: "scr_layout", options: layout_options(&layouts, None) }
                        }
                        AzFormRow { label: "绑定模型",
                            AzSelect { name: "scr_model_id", options: model_options(&models, None) }
                        }
                        AzButton { tone: AzButtonTone::Primary, button_type: "submit", "创建" }
                    }
            }
            AzTableViewport {
                AzTable { bordered: true, dense: true,
                    AzTableHead {
                        AzTableRow {
                            AzTableHeaderCell { "名称" }
                            AzTableHeaderCell { "标签" }
                            AzTableHeaderCell { "布局" }
                            AzTableHeaderCell { "绑定模型" }
                            AzTableHeaderCell { "记录数" }
                            AzTableHeaderCell { "操作" }
                        }
                    }
                    AzTableBody {
                        if screens_with_counts.is_empty() {
                            AzTableRow {
                                AzTableCell { class: "az-table__cell--empty", colspan: 6,
                                    "暂无页面"
                                }
                            }
                        } else {
                            for sc in &screens_with_counts {
                                AzTableRow {
                                    AzTableCell { code { "{sc.s.name}" } }
                                    AzTableCell { "{sc.s.label}" }
                                    AzTableCell {
                                        AzBadge { "{layout_label_from_descriptors(&layouts, &sc.s.layout)}" }
                                    }
                                    AzTableCell { "{sc.s.model_name}" }
                                    AzTableCell {
                                        AzBadge { tone: AzBadgeTone::Accent, "{sc.count}" }
                                    }
                                    AzTableCell {
                                        AzButtonLink {
                                            href: format!("/?route={lowcode_route}&screen={}", sc.s.id),
                                            "预览"
                                        }
                                        AzAccordion { title: "编辑", summary_class: "compact-summary",
                                                LowcodeActionForm {
                                                    action_name: "edit-screen",
                                                    hidden_fields: vec![("scr_id".to_string(), sc.s.id.clone())],
                                                    AzFormRow { label: "标签",
                                                        AzInput { name: "scr_label", value: sc.s.label.clone(), class: "az-input--compact" }
                                                    }
                                                    AzFormRow { label: "布局",
                                                        AzSelect { name: "scr_layout", options: layout_options(&layouts, Some(sc.s.layout.clone())) }
                                                    }
                                                    AzFormRow { label: "绑定模型",
                                                        AzSelect { name: "scr_model_id", options: model_options(&models, Some(sc.s.model_id.clone())) }
                                                    }
                                                    AzButton { tone: AzButtonTone::Primary, button_type: "submit", "保存" }
                                                }
                                        }
                                        AzButtonLink {
                                            href: format!("/?route={lowcode_route}&mode=screens&action=delete-screen&scr_id={}", sc.s.id),
                                            tone: AzButtonTone::Danger,
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
) -> Vec<AzSelectOption> {
    layouts
        .iter()
        .map(|layout| {
            AzSelectOption::new(layout.code.clone(), layout.label.clone())
                .selected(current.as_deref() == Some(layout.code.as_str()))
        })
        .collect()
}

fn model_options(models: &[MetaModelSummary], current: Option<String>) -> Vec<AzSelectOption> {
    models
        .iter()
        .map(|model| {
            AzSelectOption::new(model.id.clone(), format!("{} ({})", model.label, model.name))
                .selected(current.as_deref() == Some(model.id.as_str()))
        })
        .collect()
}
