//! Compact form primitives for dense workbench pages.

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::component_style::component_style;

fn non_empty_attr(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

/// Renders a responsive form grid for dense workbench forms.
#[allow(non_snake_case)]
#[component]
pub fn FormGrid(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] wide: bool,
) -> Element {
    let class = compose_class(
        "form-grid",
        &class,
        &[("form-grid--wide", wide), ("settings-form-grid", true)],
    );

    rsx! {
        {component_style()}
        div { class: class, {children} }
    }
}

/// Renders a GET/POST action form shell for SSR workbench actions.
#[allow(non_snake_case)]
#[component]
pub fn ActionForm(
    children: Element,
    #[props(default = "get".to_string(), into)] method: String,
    #[props(default = "/".to_string(), into)] action: String,
    #[props(default, into)] id: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
) -> Element {
    let id = non_empty_attr(id);
    let class = non_empty_attr(class);
    let style = non_empty_attr(style);

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

/// Renders a hidden input for route/action state carried by SSR forms.
#[allow(non_snake_case)]
#[component]
pub fn HiddenInput(
    #[props(into)] name: String,
    #[props(default, into)] value: String,
    #[props(default, into)] id: String,
    #[props(default, into)] class: String,
) -> Element {
    let id = non_empty_attr(id);
    let class = non_empty_attr(class);

    rsx! {
        {component_style()}
        input {
            r#type: "hidden",
            name: name,
            value: value,
            id: id,
            class: class,
        }
    }
}

/// Renders one labeled form row.
#[allow(non_snake_case)]
#[component]
pub fn FormRow(
    children: Element,
    #[props(into)] label: String,
    #[props(default, into)] class: String,
    #[props(default)] required: bool,
    #[props(default)] wide: bool,
) -> Element {
    let class = compose_class(
        "form-row settings-form-row",
        &class,
        &[("form-row--wide settings-form-row--wide", wide)],
    );

    rsx! {
        {component_style()}
        div { class: class,
            label {
                "{label}"
                if required {
                    span { class: "form-row__required", "*" }
                }
            }
            {children}
        }
    }
}

/// Renders an input with the shared workbench input style.
#[allow(non_snake_case)]
#[component]
pub fn Input(
    #[props(default = "text".to_string(), into)] input_type: String,
    #[props(default, into)] name: String,
    #[props(default, into)] value: String,
    #[props(default, into)] placeholder: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] required: bool,
) -> Element {
    let class = compose_class("form-input settings-input", &class, &[]);
    let style = non_empty_attr(style);

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

/// Select option data for [`Select`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectOption {
    /// Submitted value.
    pub value: String,
    /// Visible label.
    pub label: String,
    /// Whether the option is selected.
    pub selected: bool,
}

impl SelectOption {
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
pub fn Select(
    options: Vec<SelectOption>,
    #[props(default, into)] name: String,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] required: bool,
) -> Element {
    let class = compose_class("form-select settings-input", &class, &[]);
    let style = non_empty_attr(style);

    rsx! {
        {component_style()}
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
pub fn CheckboxRow(
    #[props(into)] label: String,
    #[props(default, into)] name: String,
    #[props(default = "1".to_string(), into)] value: String,
    #[props(default, into)] class: String,
    #[props(default)] checked: bool,
) -> Element {
    let class = compose_class("checkbox-row", &class, &[]);

    rsx! {
        {component_style()}
        label { class: class,
            input { r#type: "checkbox", name: name, value: value, checked: checked }
            "{label}"
        }
    }
}
