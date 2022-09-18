use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use derivative::Derivative;
use yew::virtual_dom::Key;

use super::ExtractKeyFn;

#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Selection2<T> {
    extract_key: ExtractKeyFn<T>,
    #[derivative(PartialEq(compare_with="selection_state_equal"))]
    inner: Rc<RefCell<SelectionState>>,
}

impl<T> Selection2<T> {

    pub fn new(multiselect: bool, extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            extract_key: extract_key.into(),
            inner: Rc::new(RefCell::new(
                SelectionState::new(multiselect)
            )),
        }
    }

    /// Clear the selection
    pub fn clear(&self) {
        self.inner.borrow_mut()
            .clear();
    }

    pub fn select<X: std::borrow::Borrow<T>>(&self, item: X) {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.select_key(key);
    }

    /// Add a key to the selection.
    pub fn select_key(&self, key: impl Into<Key>) {
        self.inner.borrow_mut()
            .select_key(key);
    }

    pub fn toggle<X: std::borrow::Borrow<T>>(&self, item: X) {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.toggle_key(key);
    }

    /// Toggle the selection state for key.
    pub fn toggle_key(&self, key: impl Into<Key>) {
        self.inner.borrow_mut()
            .toggle_key(key);
    }

    pub fn contains<X: std::borrow::Borrow<T>>(&self, item: X) -> bool {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.contains_key(&key)
    }

    // Query if the selection contains the key.
    pub fn contains_key(&self, key: &Key) -> bool {
        self.inner.borrow()
            .contains_key(key)
    }
}

fn selection_state_equal(
    me: &Rc<RefCell<SelectionState>>,
    other: &Rc<RefCell<SelectionState>>
) -> bool {
    Rc::ptr_eq(&me, &other) &&
        me.borrow().version == other.borrow().version
}

struct SelectionState {
    version: usize, // change tracking
    multiselect: bool,
    selection: Option<Key>, // used for single row
    selection_map: HashSet<Key>, // used for multiselect
}

impl SelectionState {

    fn new(multiselect: bool) -> Self {
        Self {
            version: 0,
            multiselect,
            selection: None,
            selection_map: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.version += 1;
        self.selection = None;
        self.selection_map = HashSet::new();
    }

    pub fn select_key(&mut self, key: impl Into<Key>) {
        self.version += 1;
        let key = key.into();
        match self.multiselect {
            false => self.selection = Some(key),
            true => {
                self.selection_map.insert(key);
            }
        }
    }

    pub fn toggle_key(&mut self, key: impl Into<Key>) {
        self.version += 1;
        let key = key.into();
        match self.multiselect {
            false => {
                if let Some(current) = &self.selection {
                    if current == &key {
                        self.selection = None;
                    } else {
                        self.selection = Some(key);
                    }
                } else {
                    self.selection = Some(key);
                }
            }
            true => {
                if self.selection_map.contains(&key) {
                    self.selection_map.remove(&key);
                } else {
                   self.selection_map.insert(key);
                }
            }
        }
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        match self.multiselect {
            false => {
                if let Some(current) = &self.selection {
                    if current == key {
                        return true;
                    }
                }
            }
            true => {
                if self.selection_map.contains(key) {
                    return true;
                }
            }
        }
        false
    }
}
