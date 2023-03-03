use crate::FromTransfer;
use sycamore::{prelude::*, web::html::ev};
use web_sys::DragEvent;

/// The builder for the [`create_droppable`] options
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

    /// Sets a callback to run when an item is dropped on this droppable element.
    /// The argument is parsed from the item's [`DataTransfer`](web_sys::DataTransfer).
    pub fn on_drop(mut self, f: impl Fn(T) + 'cx) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    /// A callback to check if the incoming [`DataTransfer`](web_sys::DataTransfer) should be accepted.
    /// The argument is parsed from the item's [`DataTransfer`](web_sys::DataTransfer).
    pub fn accept(mut self, f: impl Fn(&T) -> bool + 'cx) -> Self {
        self.accept = Some(Box::new(f));
        self
    }

    /// A class or list of classes to set when a valid item is hovering over the element.
    /// They are automatically removed when the item leaves or is dropped.
    pub fn hovering_class(mut self, class: impl Into<String>) -> Self {
        self.hovering_class = class.into();
        self
    }

    /// An existing [`NodeRef`] to use instead of creating a new one. Useful for combining drag and
    /// drop on one element, or using your own logic that requires [`NodeRef`].
    pub fn node_ref(mut self, node_ref: &'cx NodeRef<G>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }

    /// Create the droppable logic. Returns a [`NodeRef`] that needs to be set as the element's `ref`
    /// attribute.
    pub fn build(self) -> &'cx NodeRef<G> {
        let node = self.node_ref.unwrap_or_else(|| create_node_ref(self.scope));
        create_droppable_effect(self.scope, self, node);
        node
    }
}

/// Create a drop zone for an element. The [`DroppableBuilder`] can be used to further configure the
/// drop zone.
///
/// # Example
///
/// ```
/// # use sycamore::prelude::*;
/// # use sycamore_dnd::*;
/// #[component]
/// fn DropZone<G: Html>(cx: Scope) -> View<G> {
///     let drop = create_droppable(cx)
///         .on_drop(move |name: String| {
///             log::trace!("{name} was dropped here");
///         })
///         .build();    
///
///     view! { cx,
///         div(class = "drop-zone", ref = drop) {
///             "Drop here"
///         }
///     }
/// }
/// ```
pub fn create_droppable<G: Html, T: FromTransfer>(cx: Scope<'_>) -> DroppableBuilder<'_, G, T> {
    DroppableBuilder::new(cx)
}

fn create_droppable_effect<'cx, G: Html, T: FromTransfer + 'static>(
    cx: Scope<'cx>,
    options: DroppableBuilder<'cx, G, T>,
    node_ref: &'cx NodeRef<G>,
) {
    // SAFETY: This is safe as long as the builder has no custom `Drop` implementation
    // See documentation for `create_ref_unsafe`.
    let options = unsafe { create_ref_unsafe(cx, options) };

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
                    log::trace!("Dropping");
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
                            log::trace!("Data found and accepted, calling `on_drop`");
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
