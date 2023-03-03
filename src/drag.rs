use crate::{AsTransfer, DropEffect};
use sycamore::{prelude::*, web::html::ev};
use wasm_bindgen::JsCast;
use web_sys::{DataTransfer, DragEvent, Element};

/// The builder used to configure a draggable element
pub struct DraggableBuilder<'cx, G: Html, T: AsTransfer + 'static> {
    scope: Scope<'cx>,
    data: Option<T>,
    #[allow(clippy::type_complexity)]
    set_data: Option<Box<dyn Fn(&DataTransfer) + 'cx>>,
    dragging_class: String,
    allowed_effect: DropEffect,
    drag_image: Option<(Element, i32, i32)>,
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
            drag_image: None,
            node_ref: None,
        }
    }

    /// Sets the data that gets serialized to the [`DataTransfer`]. If you need to do custom
    /// serialization, use `set_data` instead.
    pub fn data<Data: AsTransfer>(self, data: Data) -> DraggableBuilder<'cx, G, Data> {
        DraggableBuilder {
            data: Some(data),
            dragging_class: self.dragging_class,
            allowed_effect: self.allowed_effect,
            set_data: self.set_data,
            scope: self.scope,
            node_ref: self.node_ref,
            drag_image: self.drag_image,
        }
    }

    /// Manually serialize the data to a [`DataTransfer`] object. This will be run every time the
    /// item is dragged.
    pub fn set_data(mut self, f: impl Fn(&DataTransfer) + 'cx) -> Self {
        self.set_data = Some(Box::new(f));
        self
    }

    /// Set a class or class list to be added when the element is being dragged. The class is
    /// automatically removed when the drag ends.
    pub fn dragging_class(mut self, class: impl Into<String>) -> Self {
        self.dragging_class = class.into();
        self
    }

    /// Set the [`DropEffect`] allowed when dropping the element.
    pub fn allowed_effect(mut self, effect: DropEffect) -> Self {
        self.allowed_effect = effect;
        self
    }

    /// Use an existing [`NodeRef`] instead of creating a new one. This is useful when combining
    /// drag and drop one one element, or when using custom logic that needs a [`NodeRef`].
    pub fn node_ref(mut self, node_ref: &'cx NodeRef<G>) -> Self {
        self.node_ref = Some(node_ref);
        self
    }

    /// Sets an HTML element to be shown when dragging the item. Creating this can be annoying - if
    /// an image is all that's needed use `drag_image` instead.
    pub fn drag_element(mut self, element: impl JsCast, x_offset: i32, y_offset: i32) -> Self {
        self.drag_image = Some((element.unchecked_into::<Element>(), x_offset, y_offset));
        self
    }

    /// Sets an `<img>` as a drag image with the specified source and offset. If you need more control
    /// over the element displayed, use `drag_element` instead.
    pub fn drag_image(self, src: impl AsRef<str>, x_offset: i32, y_offset: i32) -> Self {
        let image = web_sys::HtmlImageElement::new().unwrap();
        image.set_src(src.as_ref());
        self.drag_element(image, x_offset, y_offset)
    }

    /// Creates the dragging effects and returns a [`NodeRef`] that needs to be set as the `ref`
    /// attribute on the draggable element.
    pub fn build(self) -> &'cx NodeRef<G> {
        let node = self.node_ref.unwrap_or_else(|| create_node_ref(self.scope));
        create_draggable_effect(self.scope, self, node);
        node
    }
}

/// Create a draggable element. The [`DraggableBuilder`] can be used to further configure the
/// dragging behavior.
///
/// # Example
///
/// ```
/// # use sycamore::prelude::*;
/// # use sycamore_dnd::*;
/// #[component]
/// fn Item<G: Html>(cx: Scope) -> View<G> {
///     let drag = create_draggable(cx)
///         .data("Hello World!")
///         .dragging_class("opacity-20")
///         .build();
///
///     view! { cx,
///         div(class = "item", ref = drag) {
///             "Drag me"
///         }
///     }
/// }
/// ```
pub fn create_draggable<G: Html>(cx: Scope<'_>) -> DraggableBuilder<'_, G, ()> {
    DraggableBuilder::new(cx)
}

fn create_draggable_effect<'cx, G: Html, T: AsTransfer + 'static>(
    cx: Scope<'cx>,
    options: DraggableBuilder<'cx, G, T>,
    node_ref: &'cx NodeRef<G>,
) {
    // SAFETY: This is safe as long as the builder has no custom `Drop` implementation
    // See documentation for `create_ref_unsafe`.
    let options = unsafe { create_ref_unsafe(cx, options) };

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
                    if options.data.is_none() && options.set_data.is_none() {
                        ().write_to_transfer(&transfer);
                    }
                    if let Some((image, offset_x, offset_y)) = options.drag_image.as_ref() {
                        transfer.set_drag_image(image, *offset_x, *offset_y);
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
