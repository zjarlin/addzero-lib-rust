use az_dioxus_components::surface_card::SurfaceCard;
use dioxus::prelude::*;

#[test]
fn surface_card_renders_base_shell_and_children() {
    let markup = dioxus_ssr::render_element(rsx! {
        SurfaceCard { class: "surface-raised",
            "Body"
        }
    });

    assert_eq!(
        markup,
        "<article class=\"surface-card surface-raised\"><div class=\"surface-card__body\">Body</div></article>"
    );
}
