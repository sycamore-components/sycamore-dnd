// https://htmldom.dev/make-a-draggable-element/

use gloo::console::log;
use serde::{Deserialize, Serialize};
use sycadrop::{Draggable, DropEffect, DropTarget};
use sycamore::prelude::*;

use web_sys::{DataTransfer, DragEvent};

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
fn App<G: Html>(cx: Scope) -> View<G> {
    let contents = create_rc_signal(
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
        ]
        .into_iter()
        .enumerate()
        .map(|(i, item)| (create_rc_signal(i), item))
        .collect::<Vec<_>>(),
    );

    let on_drop = create_ref(cx, {
        let contents = contents.clone();
        move |target_index: RcSignal<usize>| {
            let contents = contents.clone();
            Box::new(move |transfer: DataTransfer| {
                let dragged_index: usize = transfer.get_data("text/html").unwrap().parse().unwrap();
                let target_index = *target_index.get();
                log!("Swapping {} and {}", target_index, dragged_index);
                let mut contents = contents.modify();
                contents.swap(dragged_index, target_index);
                contents
                    .get_mut(dragged_index)
                    .unwrap()
                    .0
                    .set(dragged_index);
                contents.get_mut(target_index).unwrap().0.set(target_index)
            })
        }
    });

    let contents = create_ref(cx, contents);

    view! { cx,
        div(class = "container") {
            div(class="box") {
                Keyed(
                    iterable=contents,
                    view= move |cx, (index, item)| {
                        let set_data = {
                            let index = index.clone();
                            move |transfer: DataTransfer| {
                            transfer.set_data("text/html", &index.get().to_string()).unwrap()
                        }
                    };

                        view! { cx,
                            Draggable(allowed_effect = DropEffect::Move, set_data = set_data, attr:class = "item", class_dragging = "dragging") {
                                DropTarget(
                                    on_drop = on_drop(index.clone()),
                                    class_drop_hover = "drag-over",
                                    attr:style = "width:100%;height:100%",
                                    attr:data-index = &index.get()
                                ) {
                                    (item.name)
                                }
                            }
                        }
                    },

                    key=|(_, item)| item.id,
                )
            }
        }
    }
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

    let handle_dragstart = |e: DragEvent| {
        let dom = node_ref.get::<DomNode>();
        let data_transf: DataTransfer = e.data_transfer().unwrap();
        if e.type_().contains("dragstart") {
            data_transf.set_effect_allowed("move");
            data_transf
                .set_data("text/html", &a_index.get().to_string())
                .unwrap();

            log!(format!("Transfer {:?}", &a_index.get()));
        }
        dom.set_attribute("style".into(), "opacity: 0.2".into());
    };

    let handle_dragend = |_e: DragEvent| {
        let dom = node_ref.get::<DomNode>();
        dom.set_attribute("style".into(), "opacity: 1".into());
    };
    let handle_dragenter = |_e: DragEvent| {
        let dom = node_ref.get::<DomNode>();
        dom.add_class("drag-over");
    };

    let handle_dragover = |e: DragEvent| {
        let dom = node_ref.get::<DomNode>();
        e.prevent_default();
        dom.add_class("drag-over");
    };

    let handle_dragleave = |_e: DragEvent| {
        let dom = node_ref.get::<DomNode>();
        dom.remove_class("drag-over");
    };

    let handle_drop = move |e: DragEvent| {
        let _dom = node_ref.get::<DomNode>();

        let data_transf: DataTransfer = e.data_transfer().unwrap();
        let data = data_transf.get_data("text/html").unwrap();
        log!(format!("{:?}", data));
        log!(format!("{:?}", &a_index.get()));
        let switch_item = use_context::<ItemSwitch>(cx);
        let _t = switch_item
            .contents
            .modify()
            .swap_elements(data.parse::<usize>().unwrap(), *a_index.get())
            .unwrap();
    };

    view! { cx,
        div(ref=node_ref, draggable=true, class="item", on:dragstart=handle_dragstart, on:dragend=handle_dragend, on:dragenter=handle_dragenter, on:dragover=handle_dragover, on:dragleave=handle_dragleave, on:drop=handle_drop, data-index = a_index.get()) {
            (c_item.get().name)
        }
    }
}

#[component]
fn DropZone<G: Html>(cx: Scope) -> View<G> {
    let node_ref = create_node_ref(cx);
    let item_switch = use_context::<ItemSwitch>(cx);

    let values = create_memo(cx, move || {
        let it_sw = item_switch.contents.get().as_ref().clone().items;
        let res = it_sw
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, ContentItem)>>();

        log!("{}", format!("{:?}", res));
        res
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
