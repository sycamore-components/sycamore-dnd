use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use sycamore_dnd::{create_draggable, create_droppable, DropEffect};

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
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
    let contents = create_signal(
        cx,
        vec![
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
        ],
    );

    view! { cx,
        div(class = "container") {
            div(class="box") {
                Keyed(
                    iterable=contents,
                    view= move |cx, item| view! { cx, Item(item = item, items = contents) },
                    key=|item| item.id,
                )
            }
        }
    }
}

#[component(inline_props)]
fn Item<'cx, G: Html>(
    cx: Scope<'cx>,
    item: ContentItem,
    items: &'cx Signal<Vec<ContentItem>>,
) -> View<G> {
    let node = create_draggable(cx)
        .data(item.id)
        .allowed_effect(DropEffect::Move)
        .dragging_class("dragging")
        .drag_image("/example_icon.png", 15, 15)
        .build();

    let node = create_droppable(cx)
        .node_ref(node)
        .on_drop({
            move |dragged_id: i32| {
                log::info!("Swapping {} and {}", item.id, dragged_id);
                let mut items = items.modify();
                let dragged_index = items.iter().position(|i| i.id == dragged_id).unwrap();
                let target_index = items.iter().position(|i| i.id == item.id).unwrap();
                items.swap(dragged_index, target_index);
            }
        })
        .hovering_class("drag-over")
        .build();

    view! { cx,
        div(class = "item", data-id = &item.id.to_string(), ref = node) {
            (item.name)
        }
    }
}
