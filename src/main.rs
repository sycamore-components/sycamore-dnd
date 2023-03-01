// https://htmldom.dev/make-a-draggable-element/

use gloo::{console::log};
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use wasm_bindgen::*;
use web_sys::{DataTransfer, Event};

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| {
        view! { cx,
            p { "Hello, World!" }
                ContainerWidget()
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cat {
    id: &'static str,
    name: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct ContentItem {
    id: i32,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Contents {
    items: Vec<ContentItem>,
}
impl Contents {
    pub fn swap_elements(
        &mut self,
        index1: usize,
        index2: usize,
    ) -> Option<Vec<(usize, &ContentItem)>> {
        if let (Some(_item1), Some(_item2)) = (self.items.get(index1), self.items.get(index2)) {
            self.items.swap(index1, index2);
            let updated_list = self
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| (index, item))
                .collect();
            return Some(updated_list);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemSwitch {
    contents: RcSignal<Contents>,
}

#[component]
fn ContainerWidget<G: Html>(cx: Scope) -> View<G> {
    let rc_items = create_rc_signal(Contents {
        items: vec![
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
    });

    let new_items = ItemSwitch { contents: rc_items };

    provide_context(cx, new_items);

    view! { cx,
        div(class="container") {
             DropZone{}

        }
    }
}

#[component(inline_props)]
fn DraggableItem<G: Html>(cx: Scope, a: usize, c: ContentItem) -> View<G> {
    let node_ref = create_node_ref(cx);
    let a_index = create_signal(cx, a);
    let c_item = create_signal(cx, c);

    let handle_dragstart = |e: Event| {
        let dom = node_ref.get::<DomNode>();
        let drag_event_ref: &web_sys::DragEvent = e.unchecked_ref();
        let drag_event = drag_event_ref.clone();
        let data_transf: DataTransfer = drag_event.data_transfer().unwrap();
        if e.type_().contains("dragstart") {
            data_transf.set_effect_allowed("move");
            data_transf
                .set_data("text/html", &a_index.get().to_string())
                .unwrap();

            log!(format!("Transfer {:?}", &a_index.get()));
        }
        dom.set_attribute("style", "opacity: 0.2");
    };

    let handle_dragend = |_e: Event| {
        let dom = node_ref.get::<DomNode>();
        dom.set_attribute("style", "opacity: 1");
    };
    let handle_dragenter = |_e: Event| {
        let dom = node_ref.get::<DomNode>();
        dom.add_class("drag-over");
    };

    let handle_dragover = |e: Event| {
        let dom = node_ref.get::<DomNode>();
        e.prevent_default();
        dom.add_class("drag-over");
    };

    let handle_dragleave = |_e: Event| {
        let dom = node_ref.get::<DomNode>();
        dom.remove_class("drag-over");
    };

    let handle_drop = move |e: Event| {
        let _dom = node_ref.get::<DomNode>();

        let drag_event_ref: &web_sys::DragEvent = e.unchecked_ref();
        let drag_event = drag_event_ref.clone();
        let data_transf: DataTransfer = drag_event.data_transfer().unwrap();
        let data = data_transf.get_data("text/html").unwrap();
        log!(format!("{:?}", data));
        log!(format!("{:?}", &a_index.get()));
        let switch_item = use_context::<RcSignal<ItemSwitch>>(cx);
        let sv = switch_item.get().as_ref().clone();
        let _t = sv
            .contents
            .get()
            .as_ref()
            .clone()
            .swap_elements(data.parse::<usize>().unwrap(), *a_index.get())
            .unwrap();
    };

    view! { cx,
        div(ref=node_ref, draggable=true, class="item", on:dragstart=handle_dragstart, on:dragend=handle_dragend, on:dragenter=handle_dragenter, on:dragover=handle_dragover, on:dragleave=handle_dragleave, on:drop=handle_drop) {
            (c_item.get().name)
        }
    }
}

#[component]
fn DropZone<G: Html>(cx: Scope) -> View<G> {
    let node_ref = create_node_ref(cx);
    let item_switch = use_context::<ItemSwitch>(cx);
    let item_contents = item_switch.contents.get();

    let values = create_memo(cx, move || {
        let it_sw = item_contents
            .as_ref()
            .clone()
            .items;
        it_sw
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, ContentItem)>>()
    });

    view! { cx,
        div(ref=node_ref, class="box") {
            Keyed(
                iterable=values,
                    view=|cx, (i, x)|
                        view! { cx,
                                DraggableItem(a=i, c=x )
                },
                key=|x| x.1.id,
            )
        }
    }
}
