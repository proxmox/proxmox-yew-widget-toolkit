use std::rc::Rc;
use std::cell::Cell;

/// Callback argument for menu events.
#[derive(Clone)]
pub struct MenuEvent {
    /// MenuCheckbox sets this flag
    pub checked: bool,

    keep_open: Rc<Cell<bool>>,
}

impl MenuEvent {

    pub fn new() -> Self {
        Self {
            checked: false,
            keep_open: Rc::new(Cell::new(false)),
        }
    }

    pub fn keep_open(&self, keep_open: bool) {
        self.keep_open.set(keep_open);
    }

    pub fn get_keep_open(&self) -> bool {
        self.keep_open.get()
    }
}
