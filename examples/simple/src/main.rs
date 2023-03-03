use sycamore::prelude::*;
use sycamore_dnd::{create_draggable, create_droppable};

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    sycamore::render(|cx| {
        view! { cx,
            App()
        }
    });
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let inside = create_signal(cx, false);

    let drop_inside = create_droppable(cx)
        .on_drop(move |_: ()| inside.set(true))
        .hovering_class("drag-over")
        .build();
    let drop_outside = create_droppable(cx)
        .on_drop(move |_: ()| inside.set(false))
        .hovering_class("drag-over")
        .build();
    let drag = create_draggable(cx).dragging_class("dragging").build();

    view! { cx,
        div(class = "container") {
            div(style = "min-height:100px;width:100%;", ref = drop_outside) {
                (if !*inside.get() {
                    view! { cx,
                        div(class = "item", ref = drag) {
                            "Drag me"
                        }
                    }
                } else {
                    View::empty()
                })
            }
            div(class="box", ref = drop_inside) {
                (if *inside.get() {
                    view! { cx,
                        div(class = "item", ref = drag) {
                            "Drag me"
                        }
                    }
                } else {
                    View::empty()
                })
            }
        }
    }
}
