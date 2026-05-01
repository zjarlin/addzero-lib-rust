//! IME Proxy — Headless component for web/mobile composition preedit capture.
//! Renders a transparent, floating textarea to intercept native keyboard events
//! without browser contenteditable side-effects.

use dioxus::prelude::*;

use crate::context::{CursorContext, MarkdownContext};

/// Hidden IME proxy component that catches mobile and desktop keyboard composition
/// events and synchronizes them with the `crop::Rope` AST engine.
#[component]
pub fn ImeProxy(class: Option<String>) -> Element {
    let ctx = use_context::<MarkdownContext>();
    let cursor_ctx = try_use_context::<CursorContext>();
    let editor_id = ctx.editor_id();
    let editor_id_for_sync = editor_id.clone();

    let sync_cursor = std::rc::Rc::new(move |text: String| {
        if let Some(mut c) = cursor_ctx {
            let eid = editor_id_for_sync.clone();
            spawn(async move {
                if let Some((pos, sel)) =
                    crate::context::read_cursor_and_selection(&eid, &text).await
                {
                    println!("!IMEPROXY! sync_cursor pos: {:?}", pos);
                    c.cursor_position.set(pos);
                    c.selection.set(sel);
                } else {
                    println!(
                        "!IMEPROXY! sync_cursor FAILED to read cursor for eid: {}",
                        eid
                    );
                }
            });
        }
    });

    rsx! {
        div {
            class: class.unwrap_or_default(),
            "data-md-ime-proxy-container": "true",
            style: "position: relative; width: 100%; height: 100%;",

            textarea {
                id: "{editor_id}",
                class: "nox-md-ime-proxy",
                style: "opacity: 0; position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 10; cursor: text; resize: none; color: transparent; background: transparent; caret-color: transparent;",
                value: "{ctx.raw_value()}",

                // ── Keyboard interception ──
                oninput: {
                    let sync_cursor = sync_cursor.clone();
                    move |evt: FormEvent| {
                        let text = evt.value().clone();
                        ctx.handle_value_change(text.clone());
                        ctx.trigger_parse.call(());
                        sync_cursor(text);
                    }
                },
                onkeydown: {
                    let sync_cursor = sync_cursor.clone();
                    move |_| sync_cursor(ctx.raw_value())
                },
                onkeyup: {
                    let sync_cursor = sync_cursor.clone();
                    move |_| sync_cursor(ctx.raw_value())
                },
                onclick: {
                    let sync_cursor = sync_cursor.clone();
                    move |_| sync_cursor(ctx.raw_value())
                },
                onfocus: {
                    let sync_cursor = sync_cursor.clone();
                    move |_| sync_cursor(ctx.raw_value())
                },


                // ── IME Composition (Preedit) ──
                oncompositionstart: move |_| {
                    // Lock parse triggers if needed
                },
                oncompositionupdate: move |evt| {
                    if let Some(mut c) = cursor_ctx {
                        c.preedit.set(Some(evt.data().data()));
                    }
                },
                oncompositionend: move |_| {
                    if let Some(mut c) = cursor_ctx {
                        c.preedit.set(None);
                    }
                    ctx.trigger_parse.call(());
                }
            }
        }
    }
}
