use gloo::console::log;
use sycamore::prelude::*;
use web_sys::DragEvent;

pub use web_sys::DataTransfer;

#[derive(Clone, Copy)]
pub enum DropEffect {
    None,
    Copy,
    CopyLink,
    CopyMove,
    Link,
    LinkMove,
    Move,
    All,
}

impl DropEffect {
    fn as_js(&self) -> &str {
        match self {
            DropEffect::None => "none",
            DropEffect::Copy => "copy",
            DropEffect::CopyLink => "copyLink",
            DropEffect::CopyMove => "copyMove",
            DropEffect::Link => "link",
            DropEffect::LinkMove => "linkMove",
            DropEffect::Move => "move",
            DropEffect::All => "all",
        }
    }
}

#[derive(Props)]
pub struct DraggableProps<'cx, G: Html, F: FnMut(DataTransfer) + 'cx> {
    #[prop(default)]
    class_dragging: &'static str,
    allowed_effect: DropEffect,
    set_data: F,

    attributes: Attributes<'cx, G>,
    children: Children<'cx, G>,
}

#[component]
pub fn Draggable<'cx, G: Html, F: FnMut(DataTransfer) + 'cx>(
    cx: Scope<'cx>,
    mut props: DraggableProps<'cx, G, F>,
) -> View<G> {
    let node = create_node_ref::<G>(cx);

    let children = props.children.call(cx);

    let on_drag_start = move |e: DragEvent| {
        let transfer = e.data_transfer().unwrap();
        transfer.set_effect_allowed(props.allowed_effect.as_js());
        (props.set_data)(transfer);

        node.get_raw().add_class(props.class_dragging);
        log!("Drag Started");
    };

    view! { cx,
        div(..props.attributes, ref = node, draggable = true,
            on:dragstart = on_drag_start,
            on:dragend = |_| node.get_raw().remove_class(props.class_dragging)
        ) {
            (children)
        }
    }
}

#[derive(Props)]
pub struct DropTargetProps<'cx, G: Html> {
    on_drop: Option<Box<dyn FnMut(DataTransfer) + 'cx>>,

    #[prop(default)]
    class_drop_hover: &'static str,
    attributes: Attributes<'cx, G>,
    children: Children<'cx, G>,
}

#[component]
pub fn DropTarget<'cx, G: Html>(cx: Scope<'cx>, mut props: DropTargetProps<'cx, G>) -> View<G> {
    let node = create_node_ref::<G>(cx);

    let on_drag_enter = |e: DragEvent| {
        log!("Drag entered");
        log!(e.data_transfer().unwrap().effect_allowed());
        node.get_raw().add_class(props.class_drop_hover);
    };

    let on_drag_leave = |_e: DragEvent| {
        log!("Drag left");
        node.get_raw().remove_class(props.class_drop_hover);
    };

    let on_drop = move |e: DragEvent| {
        log!("Dropped");
        if let Some(on_drop) = props.on_drop.as_mut() {
            on_drop(e.data_transfer().unwrap());
        }
        node.get_raw().remove_class(props.class_drop_hover);
    };

    let children = props.children.call(cx);

    view! { cx,
        div(..props.attributes, ref = node, on:dragenter = on_drag_enter,
            on:dragleave = on_drag_leave, on:drop = on_drop, on:dragover= |e: DragEvent| e.prevent_default()) {
            (children)
        }
    }
}
