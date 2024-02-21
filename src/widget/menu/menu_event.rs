use std::cell::Cell;
use std::rc::Rc;

/// Callback argument for menu events.
#[derive(Clone)]
pub struct MenuEvent {
    /// MenuCheckbox sets this flag
    pub checked: bool,

    keep_open: Rc<Cell<bool>>,
}

impl MenuEvent {
    pub(crate) fn new() -> Self {
        Self {
            checked: false,
            keep_open: Rc::new(Cell::new(false)),
        }
    }

    /// Set the `keep_open` flag.
    ///
    /// If set, the menu is kept open. This method can be called from
    /// inside the callback.
    pub fn keep_open(&self, keep_open: bool) {
        self.keep_open.set(keep_open);
    }

    pub(crate) fn get_keep_open(&self) -> bool {
        self.keep_open.get()
    }
}
