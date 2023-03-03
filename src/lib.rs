//! A drag and drop library for sycamore
//!
//! Adds the [`create_draggable`] and [`create_droppable`] functions that abstract the difficult
//! parts of drag and drop.
//! This library is still fairly low level and makes no assumptions about drop behaviour. This makes
//! it possible to do things like reading a file, but requires custom code for things like sortable
//! lists or other common drag and drop scenarios.
//!
//! # Example Usage
//!
//! ```
//! # use sycamore::prelude::*;
//! # use sycamore_dnd::*;
//! #[component]
//! fn App<'cx, G: Html>(cx: Scope<'cx>) -> View<G> {
//!     let inside = create_signal(cx, false);
//!
//!     let drop_inside = create_droppable(cx)
//!         .on_drop(move |_: ()| inside.set(true))
//!         .hovering_class("drag-over")
//!         .build();
//!     let drop_outside = create_droppable(cx)
//!         .on_drop(move |_: ()| inside.set(false))
//!         .hovering_class("drag-over")
//!         .build();
//!     let drag = create_draggable(cx)
//!         .dragging_class("dragging")
//!         .build();
//!
//!     view! { cx,
//!         div(class = "container") {
//!             div(style = "min-height:100px;width:100%;", ref = drop_outside) {
//!                 (if !*inside.get() {
//!                     view! { cx,
//!                         div(class = "item", ref = drag) {
//!                             "Drag me"
//!                         }
//!                     }
//!                 } else {
//!                     View::empty()
//!                 })
//!             }
//!             div(class="box", ref = drop_inside) {
//!                 (if *inside.get() {
//!                     view! { cx,
//!                         div(class = "item", ref = drag) {
//!                             "Drag me"
//!                         }
//!                     }
//!                 } else {
//!                     View::empty()
//!                 })
//!             }
//!         }
//!     }
//! }
//! ```

#![deny(missing_docs)]

use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

mod drag;
mod drop;

pub use drag::*;
pub use drop::*;
pub use web_sys::DataTransfer;

/// The effect allowed when dropping an item.
#[derive(Clone, Copy)]
pub enum DropEffect {
    /// No effect
    None,
    /// Copy the item
    Copy,
    /// Copy the item as a link
    CopyLink,
    /// Copy and move the item
    CopyMove,
    /// Link the item
    Link,
    /// Link and move the item
    LinkMove,
    /// Move the item
    Move,
    /// Any effect
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

/// A trait implemented for any value that can be written to a drag and drop [`DataTransfer`]
pub trait AsTransfer {
    /// Write the data to the [`DataTransfer`]
    fn write_to_transfer(&self, transfer: &DataTransfer);
}

impl<T: Serialize> AsTransfer for T {
    fn write_to_transfer(&self, transfer: &DataTransfer) {
        transfer
            .set_data("data/json", &serde_json::to_string(self).unwrap())
            .unwrap();
    }
}

/// A trait implemented for any value that can be read from a drag and drop [`DataTransfer`]
/// Note that to get the raw transfer you need to use [`RawTransfer`] because of limiations in Rust's
/// trait system.
pub trait FromTransfer: Sized {
    /// Read the data from the [`DataTransfer`]
    fn from_transfer(transfer: &DataTransfer) -> Option<Self>;
}

/// A wrapper type for a raw [`DataTransfer`]
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
