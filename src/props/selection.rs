use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use slab::Slab;
use yew::prelude::*;
use derivative::Derivative;
use yew::virtual_dom::Key;

use super::ExtractKeyFn;
use crate::state::optional_rc_ptr_eq;

/// Shared Selection
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct Selection2<T> {
    extract_key: ExtractKeyFn<T>,
    // Allow to store one SelectionObserver here (for convenience)
    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq"))]
    on_select: Option<Rc<SelectionObserver<T>>>,
    #[derivative(PartialEq(compare_with="selection_state_equal::<T>"))]
    inner: Rc<RefCell<SelectionState<T>>>,
}



/// Owns the selection listener callback. When dropped, the
/// listener callback will be removed fron the Selection.
pub struct SelectionObserver<T> {
    key: usize,
    inner: Rc<RefCell<SelectionState<T>>>,
}

impl<T> Drop for SelectionObserver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

impl<T> Selection2<T> {

    /// Create a new instance.
    ///
    /// Each selection requires a method to extract an unique key from
    /// the data record.
    pub fn new(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            extract_key: extract_key.into(),
            on_select: None,
            inner: Rc::new(RefCell::new(SelectionState::new())),
        }
    }

    /// Builder style method to set the multiselect flag.
    pub fn multiselect(self, multiselect: bool) -> Self {
        self.inner.borrow_mut().set_multiselect(multiselect);
        self
    }

    /// Builder style method to set the on_select callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [SelectionObserver]. The observer is stored inside the
    /// [Selection2] object, so each clone can hold a single on_select
    /// callback.
    pub fn on_select(mut self, cb: impl ::yew::html::IntoEventCallback<Selection2<T>>) -> Self {
        self.on_select = match cb.into_event_callback() {
            Some(cb) => Some(Rc::new(self.add_listener(cb))),
            None => None,
        };
        self
    }

    /// Method to add an selection observer.
    ///
    /// This is usually called by [Self::on_select], which stores the
    /// observer inside the [Selection2] object.
    pub fn add_listener(&mut self, cb: Callback<Selection2<T>>) -> SelectionObserver<T> {
        let key = self.inner.borrow_mut()
            .add_listener(cb);
        SelectionObserver { key, inner: self.inner.clone() }
    }

    fn notify_listeners(&self) {
        for (_key, listener) in self.inner.borrow().listeners.iter() {
            listener.emit(self.clone());
        }
    }

    /// Clear the selection
    pub fn clear(&self) {
        self.inner.borrow_mut()
            .clear();
        self.notify_listeners();
    }

    /// Add an item to the selection.
    pub fn select<X: std::borrow::Borrow<T>>(&self, item: X) {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.select_key(key);
    }

    /// Add a key to the selection.
    pub fn select_key(&self, key: impl Into<Key>) {
        self.inner.borrow_mut()
            .select_key(key);
        self.notify_listeners();
    }

    /// Toggle the selection state for an item.
    pub fn toggle<X: std::borrow::Borrow<T>>(&self, item: X) {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.toggle_key(key);
    }

    /// Toggle the selection state for key.
    pub fn toggle_key(&self, key: impl Into<Key>) {
        self.inner.borrow_mut()
            .toggle_key(key);
        self.notify_listeners();
    }

    /// Query if the selection contains an item.
    pub fn contains<X: std::borrow::Borrow<T>>(&self, item: X) -> bool {
        let item: &T = item.borrow();
        let key = self.extract_key.apply(item);
        self.contains_key(&key)
    }

    /// Query if the selection contains the key.
    pub fn contains_key(&self, key: &Key) -> bool {
        self.inner.borrow()
            .contains_key(key)
    }

    /// Returns all selected keys
    pub fn selected_keys(&self) -> Vec<Key> {
        self.inner.borrow()
            .selected_keys()
    }
}

impl<'a, T: 'a> Selection2<T> {
    pub fn filter_nonexistent(&self, data: impl Iterator<Item=&'a T>) {
        self.inner.borrow_mut()
            .filter_nonexistent(data, &self.extract_key);
    }
}

fn selection_state_equal<T>(
    me: &Rc<RefCell<SelectionState<T>>>,
    other: &Rc<RefCell<SelectionState<T>>>
) -> bool {
    Rc::ptr_eq(&me, &other) &&
        me.borrow().version == other.borrow().version
}

struct SelectionState<T> {
    version: usize, // change tracking
    multiselect: bool,
    selection: Option<Key>, // used for single row
    selection_map: HashSet<Key>, // used for multiselect
    listeners: Slab<Callback<Selection2<T>>>,
}

impl<T> SelectionState<T> {

    fn new() -> Self {
        Self {
            version: 0,
            multiselect: false,
            selection: None,
            selection_map: HashSet::new(),
            listeners: Slab::new(),
        }
    }

    fn set_multiselect(&mut self, multiselect: bool) {
        self.multiselect = multiselect;
    }

    fn add_listener(&mut self, cb: Callback<Selection2<T>>) -> usize {
        self.listeners.insert(cb)
    }

    fn remove_listener(&mut self, key: usize) {
        self.listeners.remove(key);
    }

    fn clear(&mut self) {
        self.version += 1;
        self.selection = None;
        self.selection_map = HashSet::new();
     }

    fn select_key(&mut self, key: impl Into<Key>) {
        self.version += 1;
        let key = key.into();
        match self.multiselect {
            false => self.selection = Some(key),
            true => {
                self.selection_map.insert(key);
            }
        }
    }

    fn toggle_key(&mut self, key: impl Into<Key>) {
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

    fn contains_key(&self, key: &Key) -> bool {
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

    fn selected_keys(&self) -> Vec<Key> {
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
}

impl<'a, T: 'a> SelectionState<T> {

    fn filter_nonexistent(
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
