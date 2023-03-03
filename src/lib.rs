use std::ops::Deref;

use serde::{de::DeserializeOwned, Serialize};

pub use web_sys::DataTransfer;

mod drag;
mod drop;

pub use drag::*;
pub use drop::*;

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
