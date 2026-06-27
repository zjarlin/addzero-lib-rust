//! Small status badges using the `status-badge` class contract.

use dioxus::prelude::*;

use crate::class_name::compose_class;
use crate::component_style::component_style;

/// Visual intent for [`StatusBadge`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StatusBadgeTone {
    /// Neutral badge.
    #[default]
    Neutral,
    /// Accent badge.
    Accent,
    /// Warning badge.
    Warn,
}

impl StatusBadgeTone {
    fn modifier(self) -> (&'static str, bool) {
        match self {
            Self::Neutral => ("", false),
            Self::Accent => ("status-badge--accent", true),
            Self::Warn => ("status-badge--warn", true),
        }
    }
}

/// Renders a compact inline badge.
#[allow(non_snake_case)]
#[component]
pub fn StatusBadge(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] tone: StatusBadgeTone,
) -> Element {
    let class = compose_class("status-badge", &class, &[tone.modifier()]);

    rsx! {
        {component_style()}
        span { class: class, {children} }
    }
}
