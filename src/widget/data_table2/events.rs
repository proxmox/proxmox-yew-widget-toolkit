use std::ops::Deref;

use yew::prelude::*;
use yew::virtual_dom::Key;

/// Like [web_sys::KeyboardEvent], but includes a record key.
pub struct DataTableKeyboardEvent {
    pub key: Key,
    pub inner: KeyboardEvent,
}

impl DataTableKeyboardEvent {

    /// Creates a new instance
    pub fn new(key: Key, inner: KeyboardEvent) -> Self {
        Self { key, inner }
    }
}

impl Deref for DataTableKeyboardEvent {
    type Target = KeyboardEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Like [web_sys::MouseEvent], but includes a record key.
pub struct DataTableMouseEvent {
    pub key: Key,
    pub inner: MouseEvent,
}

impl DataTableMouseEvent {

    /// Creates a new instance
    pub fn new(key: Key, inner: MouseEvent) -> Self {
        Self { key, inner }
    }
}

impl Deref for DataTableMouseEvent {
    type Target = MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
