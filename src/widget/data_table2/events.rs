use std::rc::Rc;
use std::ops::Deref;

use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::state::Selection2;

/// DataTable keyboard event callback.
///
/// This callback gets a mutable ref to [DataTableKeyboardEvent], so
/// that it can set the stop_propagation flag.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableKeyboardEventCallback(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableKeyboardEvent)>
);

impl DataTableKeyboardEventCallback {
    /// Creates a new instance.
    pub fn new(cb: impl 'static + Fn(&mut DataTableKeyboardEvent)) -> Self {
        Self(Rc::new(cb))
    }
    /// Emit the callback.
    pub fn emit(&self, event: &mut DataTableKeyboardEvent) {
        (self.0)(event);
    }
}

impl<F: 'static + Fn(&mut DataTableKeyboardEvent)> From<F> for DataTableKeyboardEventCallback {
    fn from(f: F) -> Self {
        DataTableKeyboardEventCallback::new(f)
    }
}

/// Like [web_sys::KeyboardEvent], but includes a record key.
pub struct DataTableKeyboardEvent {
    pub(crate) selection: Option<Selection2>,
    pub(crate) inner: KeyboardEvent,
    pub record_key: Key,
    pub(crate) stop_propagation: bool,
}

impl DataTableKeyboardEvent {
    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection2> {
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

pub trait IntoOptionalDataTableKeyboardEventCallback {
    fn into_optional_keyboard_event_cb(self) -> Option<DataTableKeyboardEventCallback>;
}

impl IntoOptionalDataTableKeyboardEventCallback for Option<DataTableKeyboardEventCallback> {
    fn into_optional_keyboard_event_cb(self) -> Option<DataTableKeyboardEventCallback> {
        self
    }
}

impl<F: 'static + Fn(&mut DataTableKeyboardEvent)> IntoOptionalDataTableKeyboardEventCallback for F {
    fn into_optional_keyboard_event_cb(self) -> Option<DataTableKeyboardEventCallback> {
        Some(DataTableKeyboardEventCallback::new(self))
    }
}

/// DataTable mouse event callback.
///
/// This callback gets a mutable ref to [DataTableMouseEvent], so
/// that it can set the stop_propagation flag.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableMouseEventCallback(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableMouseEvent)>
);

impl DataTableMouseEventCallback {
    /// Creates a new instance.
    pub fn new(cb: impl 'static + Fn(&mut DataTableMouseEvent)) -> Self {
        Self(Rc::new(cb))
    }
    /// Emit the callback.
    pub fn emit(&self, event: &mut DataTableMouseEvent) {
        (self.0)(event);
    }
}

impl<F: 'static + Fn(&mut DataTableMouseEvent)> From<F> for DataTableMouseEventCallback {
    fn from(f: F) -> Self {
        DataTableMouseEventCallback::new(f)
    }
}

/// Like [web_sys::MouseEvent], but includes a record key.
pub struct DataTableMouseEvent {
    pub(crate) selection: Option<Selection2>,
    pub(crate) inner: MouseEvent,
    pub(crate) stop_propagation: bool,
    pub record_key: Key,
}

impl DataTableMouseEvent {
    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection2> {
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

pub trait IntoOptionalDataTableMouseEventCallback {
    fn into_optional_mouse_event_cb(self) -> Option<DataTableMouseEventCallback>;
}

impl IntoOptionalDataTableMouseEventCallback for Option<DataTableMouseEventCallback> {
    fn into_optional_mouse_event_cb(self) -> Option<DataTableMouseEventCallback> {
        self
    }
}

impl<F: 'static + Fn(&mut DataTableMouseEvent)> IntoOptionalDataTableMouseEventCallback for F {
    fn into_optional_mouse_event_cb(self) -> Option<DataTableMouseEventCallback> {
        Some(DataTableMouseEventCallback::new(self))
    }
}
