use crate::FromTransfer;
use sycamore::{prelude::*, web::html::ev};
use web_sys::DragEvent;

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

    pub fn on_drop(mut self, f: impl Fn(T) + 'cx) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    pub fn accept(mut self, f: impl Fn(&T) -> bool + 'cx) -> Self {
        self.accept = Some(Box::new(f));
        self
    }

    pub fn hovering_class(mut self, class: impl Into<String>) -> Self {
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
                    log::trace!("Drag enter");
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
                    log::trace!("Drag leave");
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
                            e.prevent_default();
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
