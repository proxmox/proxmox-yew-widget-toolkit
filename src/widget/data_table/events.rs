use std::ops::Deref;

use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::state::Selection;

use super::HeaderMsg;

/// Like [web_sys::KeyboardEvent], but includes a record key.
pub struct DataTableKeyboardEvent {
    pub(crate) selection: Option<Selection>,
    pub(crate) inner: KeyboardEvent,
    pub record_key: Key,
    pub(crate) stop_propagation: bool,
}

impl DataTableKeyboardEvent {
    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    /// Stop Event propgagation
    pub fn stop_propagation(&mut self) {
        self.inner.stop_propagation();
        self.stop_propagation = true;
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
    pub(crate) selection: Option<Selection>,
    pub(crate) inner: MouseEvent,
    pub(crate) stop_propagation: bool,
    pub record_key: Key,
}

impl DataTableMouseEvent {
    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    /// Stop Event propgagation
    pub fn stop_propagation(&mut self) {
        self.inner.stop_propagation();
        self.stop_propagation = true;
    }
}

impl Deref for DataTableMouseEvent {
    type Target = MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Like [web_sys::KeyboardEvent], but allows to send header commands.
pub struct DataTableHeaderKeyboardEvent<T: 'static> {
    pub(crate) on_message: Callback<HeaderMsg<T>>,
    pub(crate) inner: KeyboardEvent,
    pub(crate) stop_propagation: bool,
}

impl<T: 'static> DataTableHeaderKeyboardEvent<T> {
    /// Stop Event propgagation
    ///
    /// If set, the header widget ignores the event.
    pub fn stop_propagation(&mut self) {
        self.inner.stop_propagation();
        self.stop_propagation = true;
    }

    /// Send command to toggle the selection state.
    pub fn send_toggle_select_all(&self) {
        self.on_message.emit(HeaderMsg::ToggleSelectAll);
    }
}

impl<T> Deref for DataTableHeaderKeyboardEvent<T> {
    type Target = KeyboardEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
