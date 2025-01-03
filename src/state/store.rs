use std::cell::{Ref, RefCell, RefMut};
use std::ops::Range;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use derivative::Derivative;
use slab::Slab;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::props::{
    ExtractKeyFn, ExtractPrimaryKey, FilterFn, IntoFilterFn, IntoSorterFn, SorterFn,
};
use crate::state::{optional_rc_ptr_eq, DataNode, DataNodeDerefGuard, DataStore};

/// Hook to use a [Store] with functional components.
///
/// This hook returns a [Store] that listens to [Store] change
/// events which trigger a redraw.
#[hook]
pub fn use_store<F, T>(init_fn: F) -> Store<T>
where
    F: FnOnce() -> Store<T>,
    T: 'static,
{
    let redraw = use_state(|| 0);

    let store = use_state(init_fn);
    let _on_change = use_state({
        let store = store.clone();
        let redraw = redraw.clone();
        move || (*store).add_listener(move |()| redraw.set(0)) // trigger redraw
    });

    (*store).clone()
}

/// Shared store for lists of records (`Vec<T>`).
///
/// Functional components can use the [use_store] hook.
///
/// # Note
///
/// A [Store] is a shared state behind `Rc<RefCell<state>>`, so
/// a simply `PartialEq` would always return true. Please register a
/// listener to get notified about changes.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct Store<T: 'static> {
    // Allow to store one StoreObserver here (for convenience)
    #[derivative(PartialEq(compare_with = "optional_rc_ptr_eq"))]
    on_change: Option<Rc<StoreObserver<T>>>,
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    inner: Rc<RefCell<StoreState<T>>>,
}

/// Owns the  listener callback. When dropped, the
/// listener callback will be removed from the [Store].
pub struct StoreObserver<T: 'static> {
    key: usize,
    inner: Rc<RefCell<StoreState<T>>>,
}

impl<T: 'static> Drop for StoreObserver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

impl<T: ExtractPrimaryKey + 'static> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ExtractPrimaryKey + 'static> Store<T> {
    /// Creates a new instance for types implementing [ExtractPrimaryKey].
    ///
    /// Use [Self::with_extract_key] for types which does not
    /// implement [ExtractPrimaryKey].
    pub fn new() -> Self {
        let extract_key = ExtractKeyFn::new(|data: &T| data.extract_key());
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(StoreState::new(extract_key))),
        }
    }
}

impl<T: 'static> Store<T> {
    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(StoreState::new(extract_key.into()))),
        }
    }

    /// Builder style method to set the on_change callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [StoreObserver]. The observer is stored inside the
    /// [Store] object, so each clone can hold a single on_select
    /// callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_change = cb
            .into_event_callback()
            .map(|cb| Rc::new(self.add_listener(cb)));
        self
    }

    /// Lock this store for read access.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably locked.
    pub fn read(&self) -> StoreReadGuard<T> {
        StoreReadGuard {
            state: self.inner.borrow(),
        }
    }

    /// Lock this store for write access.
    ///
    /// # Panics
    ///
    /// Panics if the store is already locked.
    /// When the returned [StoreWriteGuard] is dropped, the store listeners
    /// are notified. To prevent that use [StoreWriteGuard::skip_update]
    pub fn write(&self) -> StoreWriteGuard<T> {
        let state = self.inner.borrow_mut();
        StoreWriteGuard {
            state,
            update: true,
        }
    }

    // DataStore trait implementation, so that we can use those
    // methods without DataStore trait in scope.

    /// Returns the unique record key.
    pub fn extract_key(&self, data: &T) -> Key {
        self.inner.borrow().extract_key(data)
    }

    /// Method to add an change observer.
    ///
    /// This is usually called by [Self::on_change], which stores the
    /// observer inside the [Store] object.
    pub fn add_listener(&self, cb: impl Into<Callback<()>>) -> StoreObserver<T> {
        let key = self.inner.borrow_mut().add_listener(cb.into());
        StoreObserver {
            key,
            inner: self.inner.clone(),
        }
    }

    /// Set the sorter function.
    pub fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        self.write().set_sorter(sorter);
    }

    /// Set the filter function.
    pub fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        self.write().set_filter(filter);
    }

    pub fn set_data(&self, data: Vec<T>) {
        self.write().set_data(data);
    }

    pub fn clear(&self) {
        self.write().clear();
    }

    pub fn data_len(&self) -> usize {
        self.inner.borrow().data.len()
    }
    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let mut state = self.inner.borrow_mut();
        state.update_filtered_data();
        state.lookup_filtered_record_key(cursor)
    }

    pub fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        let mut state = self.inner.borrow_mut();
        state.update_filtered_data();
        state.filtered_record_pos(key)
    }

    pub fn filtered_data_len(&self) -> usize {
        let mut state = self.inner.borrow_mut();
        state.update_filtered_data();
        state.filtered_data_len()
    }

    pub fn filtered_data<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.inner.borrow_mut().update_filtered_data();
        Box::new(StoreIterator {
            range: None,
            pos: 0,
            state: self.inner.borrow(),
        })
    }

    pub fn filtered_data_range<'a>(
        &'a self,
        range: Range<usize>,
    ) -> Box<dyn Iterator<Item = (usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.inner.borrow_mut().update_filtered_data();
        Box::new(StoreIterator {
            pos: range.start,
            range: Some(range),
            state: self.inner.borrow(),
        })
    }
}

/// A wrapper type for a mutably borrowed [Store]
///
/// Notifies store listeners when dropped, except when using [StoreWriteGuard::skip_update]
pub struct StoreWriteGuard<'a, T: 'static> {
    state: RefMut<'a, StoreState<T>>,
    update: bool,
}

impl<T> StoreWriteGuard<'_, T> {
    /// Consumes the guard but don't notifies the store listeners
    /// Only use this when you did not modify the store data.
    pub fn skip_update(mut self) {
        self.update = false;
        drop(self)
    }
}

impl<T> Deref for StoreWriteGuard<'_, T> {
    type Target = StoreState<T>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T> DerefMut for StoreWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<T: 'static> Drop for StoreWriteGuard<'_, T> {
    fn drop(&mut self) {
        if self.update {
            self.version += 1;
            self.state.notify_listeners();
        }
    }
}

/// Wraps a borrowed reference to a [Store]
pub struct StoreReadGuard<'a, T> {
    state: Ref<'a, StoreState<T>>,
}

impl<T> Deref for StoreReadGuard<'_, T> {
    type Target = StoreState<T>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[doc(hidden)]
pub struct StoreNodeRef<'a, T> {
    state: Ref<'a, StoreState<T>>,
    node_id: usize,
}

impl<T: 'static> DataNode<T> for StoreNodeRef<'_, T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let data = &self.state.data[self.node_id];
        let guard: Box<dyn Deref<Target = T>> = Box::new(data);
        DataNodeDerefGuard { guard }
    }
    fn level(&self) -> usize {
        0
    }
    fn is_leaf(&self) -> bool {
        true
    }
    fn is_root(&self) -> bool {
        false
    }
    fn expanded(&self) -> bool {
        false
    }
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> {
        None
    }
    fn key(&self) -> Key {
        let record = &self.state.data[self.node_id];
        self.state.extract_key(record)
    }
}

impl<T: Clone + PartialEq + 'static> DataStore for Store<T> {
    type Observer = StoreObserver<T>;
    type Record = T;
    type Collection = Vec<T>;

    // Note: we implement all method on Store, so that we can use
    // them without DataStore trait in scope.

    fn extract_key(&self, data: &T) -> Key {
        self.inner.borrow().extract_key(data)
    }

    fn get_extract_key_fn(&self) -> ExtractKeyFn<T> {
        self.inner.borrow().get_extract_key_fn()
    }

    fn add_listener(&self, cb: impl Into<Callback<()>>) -> Self::Observer {
        let key = self.inner.borrow_mut().add_listener(cb.into());
        StoreObserver {
            key,
            inner: self.inner.clone(),
        }
    }

    fn set_data(&self, data: Self::Collection) {
        self.set_data(data);
    }

    fn clear(&self) {
        self.clear();
    }

    fn data_len(&self) -> usize {
        self.data_len()
    }

    fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        self.set_sorter(sorter);
    }

    fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        self.set_filter(filter);
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        self.lookup_filtered_record_key(cursor)
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.filtered_record_pos(key)
    }

    fn filtered_data_len(&self) -> usize {
        self.filtered_data_len()
    }

    fn filtered_data<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.filtered_data()
    }

    fn filtered_data_range<'a>(
        &'a self,
        range: Range<usize>,
    ) -> Box<dyn Iterator<Item = (usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.filtered_data_range(range)
    }
}

/// Implements the [Store] for lists of records (`Vec<T>`).
///
/// This class provides the actual [Store] implementation, and is
/// accessed via the [Store::read] and [Store::write] methods.
pub struct StoreState<T> {
    extract_key: ExtractKeyFn<T>,

    version: usize,

    data: Vec<T>,

    filtered_data: Vec<usize>,
    last_view_version: usize,

    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,

    listeners: Slab<Callback<()>>,
}

impl<T> Deref for StoreState<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for StoreState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: 'static> StoreState<T> {
    fn new(extract_key: ExtractKeyFn<T>) -> Self {
        Self {
            version: 0,
            data: Vec::new(),
            extract_key,
            filtered_data: Vec::new(),
            last_view_version: 0,
            sorter: None,
            filter: None,
            listeners: Slab::new(),
        }
    }

    /// Returns the unique record key.
    pub fn extract_key(&self, data: &T) -> Key {
        self.extract_key.apply(data)
    }

    /// Returns the primary key extraction function.
    pub fn get_extract_key_fn(&self) -> ExtractKeyFn<T> {
        self.extract_key.clone()
    }

    pub(crate) fn add_listener(&mut self, cb: Callback<()>) -> usize {
        self.listeners.insert(cb)
    }

    pub(crate) fn remove_listener(&mut self, key: usize) {
        self.listeners.remove(key);
    }

    pub(crate) fn notify_listeners(&self) {
        for (_key, listener) in self.listeners.iter() {
            listener.emit(());
        }
    }

    fn set_sorter(&mut self, sorter: impl IntoSorterFn<T>) {
        self.sorter = sorter.into_sorter_fn();
    }

    fn set_filter(&mut self, filter: impl IntoFilterFn<T>) {
        self.filter = filter.into_filter_fn();
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<T>) {
        self.data = data;
    }

    pub fn clear(&mut self) {
        self.data = Vec::new();
    }

    fn update_filtered_data(&mut self) {
        if self.version == self.last_view_version {
            return;
        }

        self.filtered_data = self
            .data
            .iter()
            .enumerate()
            .filter(|(_, record)| match &self.filter {
                Some(filter) => filter.apply(record), // fixme: remove fiter record_num param
                None => true,
            })
            .map(|(n, _record)| n)
            .collect();

        if let Some(sorter) = &self.sorter {
            self.filtered_data
                .sort_by(|a, b| sorter.cmp(&self.data[*a], &self.data[*b]));
        }
        self.last_view_version = self.version;
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let n = match self.filtered_data.get(cursor) {
            Some(n) => *n,
            None => return None,
        };

        let record = match self.data.get(n) {
            Some(record) => record,
            None => return None,
        };

        Some(self.extract_key(record))
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.filtered_data
            .iter()
            .position(|n| key == &self.extract_key(&self.data[*n]))
    }

    fn filtered_data_len(&self) -> usize {
        self.filtered_data.len()
    }

    /// Find a record position by its key.
    pub fn record_pos(&self, key: &Key) -> Option<usize> {
        self.data
            .iter()
            .position(|record| key == &self.extract_key(record))
    }

    /// Find a record by its key.
    pub fn lookup_record(&self, key: &Key) -> Option<&T> {
        self.record_pos(key).map(|n| &self.data[n])
    }

    /// Find a record by its key (mutable).
    pub fn lookup_record_mut(&mut self, key: &Key) -> Option<&mut T> {
        self.record_pos(key).map(|n| &mut self.data[n])
    }
}

#[doc(hidden)]
pub struct StoreIterator<'a, T> {
    state: Ref<'a, StoreState<T>>,
    pos: usize,
    range: Option<Range<usize>>,
}

impl<'a, T: 'static> Iterator for StoreIterator<'a, T>
where
    Self: 'a,
{
    type Item = (usize, Box<dyn DataNode<T> + 'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = &self.range {
            if range.end <= self.pos {
                return None;
            }
        }

        if self.state.filtered_data.len() <= self.pos {
            return None;
        }

        let pos = self.pos;
        self.pos += 1;

        let node_id = self.state.filtered_data[pos];
        let node = Box::new(StoreNodeRef {
            state: Ref::clone(&self.state),
            node_id,
        });

        Some((pos, node))
    }
}
