//! Compact form primitives using `az-form-*` class names.

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// Renders a responsive form grid for dense workbench forms.
#[allow(non_snake_case)]
#[component]
pub fn AzFormGrid(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] wide: bool,
) -> Element {
    let class = compose_class(
        "az-form-grid",
        &class,
        &[("az-form-grid--wide", wide), ("settings-form-grid", true)],
    );

    rsx! {
        div { class: class, {children} }
    }
}

/// Renders a GET/POST action form shell for SSR workbench actions.
#[allow(non_snake_case)]
#[component]
pub fn AzActionForm(
    children: Element,
    #[props(default = "get".to_string(), into)] method: String,
    #[props(default = "/".to_string(), into)] action: String,
    #[props(default, into)] id: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
) -> Element {
    if id.is_empty() && class.is_empty() && style.is_empty() {
        rsx! {
            form {
                method: method,
                action: action,
                {children}
            }
        }
    } else {
        rsx! {
            form {
                method: method,
                action: action,
                id: id,
                class: class,
                style: style,
                {children}
            }
        }
    }
}

/// Renders a hidden input for route/action state carried by SSR forms.
#[allow(non_snake_case)]
#[component]
pub fn AzHiddenInput(
    #[props(into)] name: String,
    #[props(default, into)] value: String,
    #[props(default, into)] id: String,
    #[props(default, into)] class: String,
) -> Element {
    if id.is_empty() && class.is_empty() {
        rsx! {
            input { r#type: "hidden", name: name, value: value }
        }
    } else {
        rsx! {
            input { r#type: "hidden", name: name, value: value, id: id, class: class }
        }
    }
}

/// Renders one labeled form row.
#[allow(non_snake_case)]
#[component]
pub fn AzFormRow(
    children: Element,
    #[props(into)] label: String,
    #[props(default, into)] class: String,
    #[props(default)] required: bool,
    #[props(default)] wide: bool,
) -> Element {
    let class = compose_class(
        "az-form-row settings-form-row",
        &class,
        &[("az-form-row--wide settings-form-row--wide", wide)],
    );

    rsx! {
        div { class: class,
            label {
                "{label}"
                if required {
                    span { class: "az-form-row__required", "*" }
                }
            }
            {children}
        }
    }
}

/// Renders an input with the shared workbench input style.
#[allow(non_snake_case)]
#[component]
pub fn AzInput(
    #[props(default = "text".to_string(), into)] input_type: String,
    #[props(default, into)] name: String,
    #[props(default, into)] value: String,
    #[props(default, into)] placeholder: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] required: bool,
) -> Element {
    let class = compose_class("az-input settings-input", &class, &[]);

    rsx! {
        input {
            class: class,
            style: style,
            r#type: input_type,
            name: name,
            value: value,
            placeholder: placeholder,
            required: required,
        }
    }
}

/// Select option data for [`AzSelect`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AzSelectOption {
    /// Submitted value.
    pub value: String,
    /// Visible label.
    pub label: String,
    /// Whether the option is selected.
    pub selected: bool,
}

impl AzSelectOption {
    /// Builds an unselected select option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            selected: false,
        }
    }

    /// Marks this option as selected when `selected` is true.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Renders a select with structured options.
#[allow(non_snake_case)]
#[component]
pub fn AzSelect(
    options: Vec<AzSelectOption>,
    #[props(default, into)] name: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] required: bool,
) -> Element {
    let class = compose_class("az-select settings-input", &class, &[]);

    rsx! {
        select { class: class, style: style, name: name, required: required,
            for option in options {
                option {
                    value: "{option.value}",
                    selected: option.selected,
                    "{option.label}"
                }
            }
        }
    }
}

/// Renders a compact checkbox row.
#[allow(non_snake_case)]
#[component]
pub fn AzCheckboxRow(
    #[props(into)] label: String,
    #[props(default, into)] name: String,
    #[props(default = "1".to_string(), into)] value: String,
    #[props(default, into)] class: String,
    #[props(default)] checked: bool,
) -> Element {
    let class = compose_class("az-checkbox-row", &class, &[]);

    rsx! {
        label { class: class,
            input { r#type: "checkbox", name: name, value: value, checked: checked }
            "{label}"
        }
    }
}
