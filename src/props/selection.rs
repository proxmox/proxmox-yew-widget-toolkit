use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use yew::prelude::*;
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

    pub fn new(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            extract_key: extract_key.into(),
            inner: Rc::new(RefCell::new(SelectionState::new())),
        }
    }

    pub fn multiselect(self, multiselect: bool) -> Self {
        self.inner.borrow_mut().set_multiselect(multiselect);
        self
    }

    pub fn on_select(self, cb: impl ::yew::html::IntoEventCallback<Vec<Key>>) -> Self {
        self.inner.borrow_mut()
            .add_listener(cb);
        self
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

impl<'a, T: 'a> Selection2<T> {
    pub fn filter_nonexistent(&self, data: impl Iterator<Item=&'a T>) {
        self.inner.borrow_mut()
            .filter_nonexistent(data, &self.extract_key);
    }
}

fn selection_state_equal(
    me: &Rc<RefCell<SelectionState>>,
    other: &Rc<RefCell<SelectionState>>
) -> bool {
    Rc::ptr_eq(&me, &other) &&
        me.borrow().version == other.borrow().version
}

enum SelectionListeners {
    None,
    Single(Callback<Vec<Key>>),
    Multiple(Vec<Callback<Vec<Key>>>),
}

struct SelectionState {
    version: usize, // change tracking
    multiselect: bool,
    selection: Option<Key>, // used for single row
    selection_map: HashSet<Key>, // used for multiselect
    on_select: SelectionListeners,
}

impl SelectionState {

    fn new() -> Self {
        Self {
            version: 0,
            multiselect: false,
            selection: None,
            selection_map: HashSet::new(),
            on_select: SelectionListeners::None,
        }
    }

    fn set_multiselect(&mut self, multiselect: bool) {
        self.multiselect = multiselect;
    }

    fn add_listener(&mut self, cb: impl ::yew::html::IntoEventCallback<Vec<Key>>) {
        if let Some(cb) = cb.into_event_callback() {
            self.on_select = match &self.on_select {
                SelectionListeners::None => SelectionListeners::Single(cb),
                SelectionListeners::Single(prev) => {
                    SelectionListeners::Multiple(vec![prev.clone(), cb])
                }
                SelectionListeners::Multiple(list) => {
                    let mut list = list.clone();
                    list.push(cb);
                    SelectionListeners::Multiple(list)
                }
            };
        }
    }

    pub fn notify_listeners(&mut self) {
        match &self.on_select {
            SelectionListeners::None => { /* do nothing */ },
            SelectionListeners::Single(cb) => {
                 cb.emit(self.selected_keys());
            }
            SelectionListeners::Multiple(list) => {
                for cb in list { cb.emit(self.selected_keys()); }
            }
        }
    }

    pub fn clear(&mut self) {
        self.version += 1;
        self.selection = None;
        self.selection_map = HashSet::new();
        self.notify_listeners();
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
        self.notify_listeners();
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
        self.notify_listeners();
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

    pub fn selected_keys(&self) -> Vec<Key> {
        let mut keys = Vec::new();

        match self.multiselect {
            false => {
                if let Some(selection) = &self.selection {
                    keys.push(selection.clone());
                }
            }
            true => {
                keys = self.selection_map.iter().map(Key::clone).collect();
            }
        }

        keys
    }

    fn filter_nonexistent<'a, T: 'a>(
        &mut self,
        mut data: impl Iterator<Item=&'a T>,
        extract_key: &ExtractKeyFn<T>,
    ) {
        match self.multiselect {
            false => {
                if let Some(current) = &self.selection {
                    self.selection = data.find_map(move |rec| {
                        let key = extract_key.apply(&rec);
                        (&key == current).then(|| key)
                    });
                }
            }
            true => {
                let mut new_map = HashSet::new();
                for rec in data {
                    let key = extract_key.apply(&rec);
                    if self.contains_key(&key) {
                        new_map.insert(key);
                    }
                }
                self.selection_map = new_map;
            }
        }
    }
}
