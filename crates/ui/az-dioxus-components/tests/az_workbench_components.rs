use az_dioxus_components::{
    az_accordion::AzAccordion,
    az_badge::{AzBadge, AzBadgeTone},
    az_button::{AzButton, AzButtonLink, AzButtonTone},
    az_form::{
        AzActionForm, AzCheckboxRow, AzFormGrid, AzFormRow, AzHiddenInput, AzInput, AzSelect,
        AzSelectOption,
    },
    az_workbench::{
        AzPageHeader, AzSplitWorkbench, AzTableViewport, AzToolbar, AzWorkbenchDetail,
        AzWorkbenchDetailHeader, AzWorkbenchPage, AzWorkbenchTree, AzWorkbenchTreeHeader,
        AzWorkbenchTreeList,
    },
};
use dioxus::prelude::*;

#[test]
fn az_workbench_components_render_shared_shell_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzWorkbenchPage {
            AzPageHeader { title: "低代码", subtitle: "元数据",
                AzToolbar {
                    AzButtonLink { href: "/?route=/lowcode", "返回" }
                }
            }
            AzSplitWorkbench {
                AzWorkbenchTree {
                    AzWorkbenchTreeHeader { title: "模型" }
                    AzWorkbenchTreeList { "Project" }
                }
                AzWorkbenchDetail {
                    AzWorkbenchDetailHeader { title: "字段", subtitle: "Project" }
                    AzTableViewport { "table" }
                }
            }
        }
    });

    assert!(markup.contains("az-workbench-page lowcode-page"));
    assert!(markup.contains("az-page-header lowcode-page__header"));
    assert!(markup.contains("az-split-workbench lowcode-workbench"));
    assert!(markup.contains("az-table-viewport lowcode-table-scroll"));
}

#[test]
fn az_form_components_render_inputs_and_options() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzFormGrid {
            AzFormRow { label: "名称", required: true,
                AzInput { name: "name", placeholder: "Project", required: true }
            }
            AzFormRow { label: "类型",
                AzSelect {
                    name: "kind",
                    options: vec![
                        AzSelectOption::new("String", "字符串"),
                        AzSelectOption::new("Integer", "整数").selected(true),
                    ],
                }
            }
            AzCheckboxRow { name: "required", label: "必填", checked: true }
            AzActionForm {
                id: "demo-form",
                AzHiddenInput { name: "route", value: "/lowcode" }
            }
        }
    });

    assert!(markup.contains("az-form-grid"));
    assert!(markup.contains("az-form-row settings-form-row"));
    assert!(markup.contains("class=\"az-input settings-input\""));
    assert!(markup.contains("value=\"Integer\""));
    assert!(markup.contains("selected"));
    assert!(markup.contains(">整数</option>"));
    assert!(markup.contains("class=\"az-checkbox-row\""));
    assert!(markup.contains(r#"<form method="get" action="/" id="demo-form" class="" style="">"#));
    assert!(markup.contains(r#"<input type="hidden" name="route" value="/lowcode"/>"#));
}

#[test]
fn az_action_components_render_tone_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzToolbar {
            AzButton { tone: AzButtonTone::Primary, button_type: "submit", "保存" }
            AzButtonLink { href: "/danger", tone: AzButtonTone::Danger, "删除" }
            AzBadge { "普通" }
            AzBadge { tone: AzBadgeTone::Accent, "高亮" }
            AzBadge { tone: AzBadgeTone::Warn, "警告" }
            AzAccordion { title: "编辑", open: true, "body" }
        }
    });

    assert!(markup.contains("az-button toolbar-button az-button--primary toolbar-button--primary"));
    assert!(markup.contains("az-button toolbar-button az-button--danger toolbar-button--danger"));
    assert!(markup.contains("class=\"az-badge\""));
    assert!(markup.contains("az-badge az-badge--accent"));
    assert!(markup.contains("az-accordion lowcode-accordion"));
}
