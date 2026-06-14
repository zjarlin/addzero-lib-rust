// Field-type-aware inline editor. Renders the appropriate widget based on MetaFieldView's field_type.
use crate::model::MetaFieldView;
use adui_dioxus::components::input::Input;
use adui_dioxus::components::input_number::InputNumber;
use adui_dioxus::components::switch::Switch;
use adui_dioxus::components::select::{Select, SelectMode};
use adui_dioxus::components::select_base::SelectOption;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum RelationValue {
    Id(String),
    Label(String),
}

#[derive(Props, Clone, PartialEq)]
pub struct FieldEditorProps {
    pub field: MetaFieldView,
    #[props(optional)]
    pub value: Option<String>,
    #[props(optional)]
    pub placeholder: Option<String>,
    pub on_change: EventHandler<(String, String)>,
    #[props(optional)]
    pub relation_options: Option<Vec<(String, String)>>,
}

pub fn FieldEditor(props: FieldEditorProps) -> Element {
    let ft = props.field.field_type.as_str();
    let current = props.value.clone().unwrap_or_default();
    let ph = props.placeholder.clone().unwrap_or_else(|| props.field.label.clone());
    let fname = props.field.name.clone();

    match ft {
        "Boolean" => {
            let checked = matches!(current.as_str(), "true" | "1" | "yes");
            rsx! {
                div { style: "display:flex; align-items:center; gap:8px;",
                    Switch {
                        checked,
                        on_change: {
                            let fname2 = fname.clone();
                            Some(EventHandler::new(move |v: bool| {
                                props.on_change.call((fname2.clone(), v.to_string()));
                            }))
                        },
                    }
                    span { style: "font-size:12px; color:var(--adui-color-text-secondary);", if checked { "是" } else { "否" } }
                }
            }
        }
        "Integer" | "Float" => {
            let val: Option<f64> = current.parse().ok();
            rsx! {
                div { style: "min-width:140px;",
                    InputNumber {
                        value: val,
                        on_change: {
                            let fname2 = fname.clone();
                            Some(EventHandler::new(move |v: Option<f64>| {
                                let s = match v {
                                    Some(n) => if ft == "Integer" { (n as i64).to_string() } else { n.to_string() },
                                    None => String::new(),
                                };
                                props.on_change.call((fname2.clone(), s));
                            }))
                        },
                    }
                }
            }
        }
        "DateTime" => {
            rsx! {
                input {
                    r#type: "date",
                    style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); min-width:160px;",
                    value: "{current}",
                    oninput: {
                        let fname2 = fname.clone();
                        move |ev: FormEvent| props.on_change.call((fname2.clone(), ev.value()))
                    },
                }
            }
        }
        "Json" => {
            rsx! {
                textarea {
                    style: "padding:6px 10px; border:1px solid var(--adui-color-border); border-radius:6px; font-size:12px; background:var(--adui-color-bg-container); color:var(--adui-color-text); min-width:200px; min-height:60px; resize:vertical; font-family:monospace;",
                    rows: "3",
                    value: "{current}",
                    oninput: {
                        let fname2 = fname.clone();
                        move |ev: FormEvent| props.on_change.call((fname2.clone(), ev.value()))
                    },
                }
            }
        }
        "Relation" => {
            if let Some(ref opts) = props.relation_options {
                let options: Vec<SelectOption> = std::iter::once(SelectOption {
                    value: "".into(),
                    label: "— 不选 —".into(),
                }).chain(opts.iter().map(|(id, label)| SelectOption {
                    value: id.clone(),
                    label: label.clone(),
                })).collect();
                rsx! {
                    div { style: "min-width:180px;",
                        Select {
                            value: if current.is_empty() { None } else { Some(current.clone()) },
                            options,
                            mode: SelectMode::Single,
                            allow_clear: true,
                            placeholder: Some(ph),
                            on_change: {
                                let fname2 = fname.clone();
                                Some(EventHandler::new(move |v: String| {
                                    props.on_change.call((fname2.clone(), v));
                                }))
                            },
                        }
                    }
                }
            } else {
                rsx! {
                    Input {
                        placeholder: Some(ph),
                        value: current,
                        on_change: {
                            let fname2 = fname.clone();
                            move |v: String| props.on_change.call((fname2.clone(), v))
                        },
                    }
                }
            }
        }
        _ => {
            rsx! {
                Input {
                    placeholder: Some(ph),
                    value: current,
                    on_change: {
                        let fname2 = fname.clone();
                        move |v: String| props.on_change.call((fname2.clone(), v))
                    },
                }
            }
        }
    }
}
