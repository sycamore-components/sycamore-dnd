use std::ops::Deref;

use gloo::console::log;
use serde::{de::DeserializeOwned, Serialize};
use sycamore::{prelude::*, web::html::ev};
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

impl Default for DropEffect {
    fn default() -> Self {
        // This is called `uninitialized` in JS, but behaves the same as `all`
        DropEffect::All
    }
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

pub trait AsTransfer {
    fn write_to_transfer(&self, transfer: &DataTransfer);
}

impl<T: Serialize> AsTransfer for T {
    fn write_to_transfer(&self, transfer: &DataTransfer) {
        transfer
            .set_data("data/json", &serde_json::to_string(self).unwrap())
            .unwrap();
    }
}

pub trait FromTransfer: Sized {
    fn from_transfer(transfer: &DataTransfer) -> Option<Self>;
}

pub struct RawTransfer(DataTransfer);

impl Deref for RawTransfer {
    type Target = DataTransfer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromTransfer for RawTransfer {
    fn from_transfer(transfer: &DataTransfer) -> Option<Self> {
        Some(RawTransfer(transfer.clone()))
    }
}

impl<T: DeserializeOwned> FromTransfer for T {
    fn from_transfer(transfer: &DataTransfer) -> Option<Self> {
        let data = transfer.get_data("data/json").ok()?;
        serde_json::from_str(&data).ok()
    }
}

pub struct DraggableBuilder<'cx, G: Html, T: AsTransfer + 'static> {
    scope: Scope<'cx>,
    data: Option<T>,
    #[allow(clippy::type_complexity)]
    set_data: Option<Box<dyn Fn(&DataTransfer) + 'cx>>,
    dragging_class: String,
    allowed_effect: DropEffect,
    node_ref: Option<&'cx NodeRef<G>>,
}

impl<'cx, G: Html, T: AsTransfer> DraggableBuilder<'cx, G, T> {
    fn new(scope: Scope<'cx>) -> Self {
        Self {
            scope,
            data: None,
            set_data: None,
            dragging_class: Default::default(),
            allowed_effect: Default::default(),
            node_ref: None,
        }
    }

    pub fn data<Data: AsTransfer>(self, data: Data) -> DraggableBuilder<'cx, G, Data> {
        DraggableBuilder {
            data: Some(data),
            dragging_class: self.dragging_class,
            allowed_effect: self.allowed_effect,
            set_data: self.set_data,
            scope: self.scope,
            node_ref: self.node_ref,
        }
    }

    pub fn set_data<F: Fn(&DataTransfer) + 'cx>(mut self, f: F) -> Self {
        self.set_data = Some(Box::new(f));
        self
    }

    pub fn dragging_class<S: Into<String>>(mut self, class: S) -> Self {
        self.dragging_class = class.into();
        self
    }

    pub fn allowed_effect(mut self, effect: DropEffect) -> Self {
        self.allowed_effect = effect;
        self
    }

    pub fn node_ref(mut self, node_ref: &'cx NodeRef<G>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }

    pub fn build(self) -> &'cx NodeRef<G> {
        let node = self.node_ref.unwrap_or_else(|| create_node_ref(self.scope));
        create_draggable_effect(self.scope, self, node);
        node
    }
}

pub fn create_draggable<G: Html, T: AsTransfer + 'static>(
    cx: Scope<'_>,
) -> DraggableBuilder<'_, G, ()> {
    DraggableBuilder::new(cx)
}

pub struct DroppableBuilder<'cx, G: Html, T: FromTransfer + 'static = ()> {
    scope: Scope<'cx>,
    on_drop: Option<Box<dyn Fn(T) + 'cx>>,
    #[allow(clippy::type_complexity)]
    accept: Option<Box<dyn Fn(&T) -> bool + 'cx>>,
    hovering_class: String,
    node_ref: Option<&'cx NodeRef<G>>,
}

impl<'cx, G: Html, T: FromTransfer + 'static> DroppableBuilder<'cx, G, T> {
    fn new(scope: Scope<'cx>) -> Self {
        Self {
            scope,
            on_drop: None,
            accept: None,
            hovering_class: Default::default(),
            node_ref: None,
        }
    }

    pub fn on_drop<F: Fn(T) + 'cx>(mut self, f: F) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    pub fn accept<F: Fn(&T) -> bool + 'cx>(mut self, f: F) -> Self {
        self.accept = Some(Box::new(f));
        self
    }

    pub fn hovering_class<S: Into<String>>(mut self, class: S) -> Self {
        self.hovering_class = class.into();
        self
    }

    pub fn node_ref(mut self, node_ref: &'cx NodeRef<G>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }

    pub fn build(self) -> &'cx NodeRef<G> {
        let node = self.node_ref.unwrap_or_else(|| create_node_ref(self.scope));
        create_droppable_effect(self.scope, self, node);
        node
    }
}

pub fn create_droppable<G: Html, T: FromTransfer>(cx: Scope<'_>) -> DroppableBuilder<'_, G, T> {
    DroppableBuilder::new(cx)
}

pub fn create_draggable_effect<'cx, G: Html, T: AsTransfer + 'static>(
    cx: Scope<'cx>,
    options: DraggableBuilder<'cx, G, T>,
    node_ref: &'cx NodeRef<G>,
) {
    // SAFETY: This is only needed because of limitations in Rust's type system. The lifetime of
    // this reference is `'cx` anyways, so casting the builder to be `'static` is safe.
    let options = create_ref(cx, unsafe {
        std::mem::transmute::<_, DraggableBuilder<'static, G, T>>(options)
    });

    create_effect(cx, move || {
        if let Some(node) = node_ref.try_get_raw() {
            let on_drag_start = {
                let node = node.clone();
                move |e: DragEvent| {
                    log!("Drag start", format!("{node:?}"));

                    let transfer = e.data_transfer().unwrap();
                    transfer.set_effect_allowed(options.allowed_effect.as_js());
                    if let Some(data) = options.data.as_ref() {
                        data.write_to_transfer(&transfer);
                    }
                    if let Some(set_data) = options.set_data.as_ref() {
                        set_data(&transfer);
                    }

                    node.add_class(&options.dragging_class);
                    node.set_attribute("data-dragging".into(), "".into());
                }
            };

            let on_drag_end = {
                let node = node.clone();
                move |_: DragEvent| {
                    log!("Drag end", format!("{node:?}"));

                    node.remove_class(&options.dragging_class);
                    node.remove_attribute("data-dragging".into());
                }
            };

            node.set_attribute("draggable".into(), "true".into());
            node.event(cx, ev::dragstart, on_drag_start);
            node.event(cx, ev::dragend, on_drag_end);
        }
    });
}

pub fn create_droppable_effect<'cx, G: Html, T: FromTransfer + 'static>(
    cx: Scope<'cx>,
    options: DroppableBuilder<'cx, G, T>,
    node_ref: &'cx NodeRef<G>,
) {
    // SAFETY: This is only needed because of limitations in Rust's type system. The lifetime of
    // this reference is `'cx` anyways, so casting the builder to be `'static` is safe.
    let options = create_ref(cx, unsafe {
        std::mem::transmute::<_, DroppableBuilder<'static, G, T>>(options)
    });

    create_effect(cx, move || {
        if let Some(node) = node_ref.try_get_raw() {
            let on_drag_enter = {
                let node = node.clone();
                move |e: DragEvent| {
                    log!("Drag enter", format!("{node:?}"));
                    e.prevent_default();

                    let should_accept = options
                        .accept
                        .as_ref()
                        .map(|accept| {
                            if let Some(data) = T::from_transfer(&e.data_transfer().unwrap()) {
                                accept(&data)
                            } else {
                                false
                            }
                        })
                        .unwrap_or(true);
                    if should_accept {
                        node.add_class(&options.hovering_class);
                    }
                }
            };

            let on_drag_leave = {
                let node = node.clone();
                move |e: DragEvent| {
                    e.prevent_default();

                    node.remove_class(&options.hovering_class);
                    log!("Drag leave", format!("{node:?}"));
                }
            };

            let on_drag_over = |e: DragEvent| {
                let should_accept = if let Some(accept) = options.accept.as_ref() {
                    if let Some(data) = T::from_transfer(&e.data_transfer().unwrap()) {
                        accept(&data)
                    } else {
                        false
                    }
                } else {
                    true
                };
                if should_accept {
                    e.prevent_default();
                }
            };

            let on_drop = {
                let node = node.clone();
                move |e: DragEvent| {
                    node.remove_class(&options.hovering_class);

                    if let Some((on_drop, data)) = options
                        .on_drop
                        .as_ref()
                        .zip(T::from_transfer(&e.data_transfer().unwrap()))
                    {
                        if options
                            .accept
                            .as_ref()
                            .map(|accept| accept(&data))
                            .unwrap_or(true)
                        {
                            on_drop(data);
                        }
                    }
                }
            };

            node.event(cx, ev::dragenter, on_drag_enter);
            node.event(cx, ev::dragleave, on_drag_leave);
            node.event(cx, ev::dragover, on_drag_over);
            node.event(cx, ev::drop, on_drop);
        }
    });
}