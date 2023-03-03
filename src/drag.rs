use crate::{AsTransfer, DropEffect};
use sycamore::{prelude::*, web::html::ev};
use web_sys::{DataTransfer, DragEvent};

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
                    log::trace!("Drag start");

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
                    log::trace!("Drag end");

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
