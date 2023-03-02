// https://htmldom.dev/make-a-draggable-element/

use gloo::console::log;
use serde::{Deserialize, Serialize};
use sycadrop::{DataTransfer, Draggable, DropEffect, DropTarget};
use sycamore::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| {
        view! { cx,
            p { "Hello, World!" }
            App()
        }
    });
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct ContentItem {
    id: i32,
    name: String,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let contents = create_rc_signal(vec![
        ContentItem {
            id: 0,
            name: "Test item 0".to_string(),
        },
        ContentItem {
            id: 1,
            name: "Test item 1".to_string(),
        },
        ContentItem {
            id: 2,
            name: "Test item 2".to_string(),
        },
        ContentItem {
            id: 3,
            name: "Test item 3".to_string(),
        },
    ]);

    let on_drop = create_ref(cx, {
        let contents = contents.clone();
        move |target_id: i32| {
            let contents = contents.clone();
            Box::new(move |transfer: DataTransfer| {
                let dragged_id: i32 = transfer.get_data("text/html").unwrap().parse().unwrap();
                log!("Swapping {} and {}", target_id, dragged_id);
                let (dragged_index, target_index) = {
                    let contents = contents.get();
                    (
                        contents.iter().position(|i| i.id == dragged_id).unwrap(),
                        contents.iter().position(|i| i.id == target_id).unwrap(),
                    )
                };
                let mut contents = contents.modify();
                contents.swap(dragged_index, target_index);
            })
        }
    });

    let contents = create_ref(cx, contents);

    view! { cx,
        div(class = "container") {
            div(class="box") {
                Keyed(
                    iterable=contents,
                    view= move |cx, item| {
                        let set_data =
                            move |transfer: DataTransfer| {
                            transfer.set_data("text/html", &item.id.to_string()).unwrap();
                    };

                        view! { cx,
                            Draggable(allowed_effect = DropEffect::Move, set_data = set_data, attr:class = "item", class_dragging = "dragging") {
                                DropTarget(
                                    on_drop = on_drop(item.id),
                                    class_drop_hover = "drag-over",
                                    attr:style = "width:100%;height:100%",
                                    attr:data-id = &item.id.to_string()
                                ) {
                                    (item.name)
                                }
                            }
                        }
                    },

                    key=|item| item.id,
                )
            }
        }
    }
}
