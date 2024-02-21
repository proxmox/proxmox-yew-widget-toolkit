use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;

use web_sys::MouseEvent;

/// Callback argument for SlidableAction mouse events.
#[derive(Clone)]
pub struct SlidableActionMouseEvent {
    inner: MouseEvent,
    keep_open: Rc<Cell<bool>>,
    dismiss: Rc<Cell<bool>>,
}

impl Deref for SlidableActionMouseEvent {
    type Target = MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl SlidableActionMouseEvent {
    pub(crate) fn new(event: MouseEvent) -> Self {
        Self {
            inner: event,
            keep_open: Rc::new(Cell::new(false)),
            dismiss: Rc::new(Cell::new(false)),
        }
    }

    /// Set the `keep_open` flag.
    ///
    /// If set, the slidable is kept open. This method can be called from
    /// inside the callback.
    pub fn keep_open(&self, keep_open: bool) {
        self.keep_open.set(keep_open);
    }

    pub(crate) fn get_keep_open(&self) -> bool {
        self.keep_open.get()
    }

    /// Set the `dismiss` flag.
    ///
    /// If set, the slidable is dismissed. This method can be called from
    /// inside the callback.
    pub fn dismiss(&self, dismiss: bool) {
        self.dismiss.set(dismiss);
    }

    pub(crate) fn get_dismiss(&self) -> bool {
        self.dismiss.get()
    }
}
