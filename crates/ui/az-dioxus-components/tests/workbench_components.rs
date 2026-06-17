use az_dioxus_components::{
    accordion::Accordion,
    form::{ActionForm, CheckboxRow, FormGrid, FormRow, HiddenInput, Input, Select, SelectOption},
    status_badge::{StatusBadge, StatusBadgeTone},
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    workbench::{
        PageHeader, SplitWorkbench, TableViewport, Toolbar, WorkbenchDetail, WorkbenchDetailHeader,
        WorkbenchPage, WorkbenchTree, WorkbenchTreeHeader, WorkbenchTreeList,
    },
};
use dioxus::prelude::*;

#[test]
fn workbench_components_render_shared_shell_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        WorkbenchPage {
            PageHeader { title: "低代码", subtitle: "元数据",
                Toolbar {
                    ToolbarButtonLink { href: "/?route=/lowcode", "返回" }
                }
            }
            SplitWorkbench {
                WorkbenchTree {
                    WorkbenchTreeHeader { title: "模型" }
                    WorkbenchTreeList { "Project" }
                }
                WorkbenchDetail {
                    WorkbenchDetailHeader { title: "字段", subtitle: "Project" }
                    TableViewport { "table" }
                }
            }
        }
    });

    assert!(markup.contains("workbench-page lowcode-page"));
    assert!(markup.contains("page-header lowcode-page__header"));
    assert!(markup.contains("split-workbench lowcode-workbench"));
    assert!(markup.contains("table-view-viewport lowcode-table-scroll"));
}

#[test]
fn form_components_render_inputs_and_options() {
    let markup = dioxus_ssr::render_element(rsx! {
        FormGrid {
            FormRow { label: "名称", required: true,
                Input { name: "name", placeholder: "Project", required: true }
            }
            FormRow { label: "类型",
                Select {
                    name: "kind",
                    options: vec![
                        SelectOption::new("String", "字符串"),
                        SelectOption::new("Integer", "整数").selected(true),
                    ],
                }
            }
            CheckboxRow { name: "required", label: "必填", checked: true }
            ActionForm {
                id: "demo-form",
                HiddenInput { name: "route", value: "/lowcode" }
            }
        }
    });

    assert!(markup.contains("form-grid"));
    assert!(markup.contains("form-row settings-form-row"));
    assert!(markup.contains("class=\"form-input settings-input\""));
    assert!(markup.contains("value=\"Integer\""));
    assert!(markup.contains("selected"));
    assert!(markup.contains(">整数</option>"));
    assert!(markup.contains("class=\"checkbox-row\""));
    assert!(markup.contains(r#"<form method="get" action="/" id="demo-form">"#));
    assert!(!markup.contains(r#"class="""#));
    assert!(!markup.contains(r#"style="""#));
    assert!(markup.contains(r#"<input type="hidden" name="route" value="/lowcode"/>"#));
}

#[test]
fn action_components_render_tone_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        Toolbar {
            ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "保存" }
            ToolbarButtonLink { href: "/danger", tone: ToolbarButtonTone::Danger, "删除" }
            StatusBadge { "普通" }
            StatusBadge { tone: StatusBadgeTone::Accent, "高亮" }
            StatusBadge { tone: StatusBadgeTone::Warn, "警告" }
            Accordion { title: "编辑", open: true, "body" }
        }
    });

    assert!(markup.contains("toolbar-button toolbar-button--primary"));
    assert!(markup.contains("toolbar-button toolbar-button--danger"));
    assert!(markup.contains("class=\"status-badge\""));
    assert!(markup.contains("status-badge status-badge--accent"));
    assert!(markup.contains("accordion lowcode-accordion"));
}
