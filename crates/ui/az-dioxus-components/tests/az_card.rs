use az_dioxus_components::az_card::AzCard;
use dioxus::prelude::*;

#[test]
fn az_card_renders_base_shell_and_children() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzCard { class: "surface-raised",
            "Body"
        }
    });

    assert_eq!(
        markup,
        "<article class=\"az-card surface-raised\"><div class=\"az-card__body\">Body</div></article>"
    );
}
