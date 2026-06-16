//! Small status badges using the `az-badge` class contract.

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// Visual intent for [`AzBadge`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AzBadgeTone {
    /// Neutral badge.
    #[default]
    Neutral,
    /// Accent badge.
    Accent,
    /// Warning badge.
    Warn,
}

impl AzBadgeTone {
    fn modifier(self) -> (&'static str, bool) {
        match self {
            Self::Neutral => ("", false),
            Self::Accent => ("az-badge--accent", true),
            Self::Warn => ("az-badge--warn", true),
        }
    }
}

/// Renders a compact inline badge.
#[allow(non_snake_case)]
#[component]
pub fn AzBadge(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] tone: AzBadgeTone,
) -> Element {
    let class = compose_class("az-badge", &class, &[tone.modifier()]);

    rsx! {
        span { class: class, {children} }
    }
}
