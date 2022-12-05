#[macro_use]
mod slab_tree;
pub use slab_tree::{SlabTree, SlabTreeNodeMut, SlabTreeNodeRef, SlabTreeChildren, SlabTreeChildrenMut};
pub(crate) use slab_tree::SlabTreeEntry;

mod keyed_slab_tree;
pub use keyed_slab_tree::{
    KeyedSlabTree, KeyedSlabTreeNodeMut, KeyedSlabTreeNodeRef,
    KeyedSlabTreeChildren, KeyedSlabTreeChildrenMut,
};

mod slab_tree_serde;

use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut, Range};

use derivative::Derivative;

use yew::virtual_dom::Key;
use yew::prelude::*;
use yew::html::IntoEventCallback;

use crate::props::{ExtractKeyFn, ExtractPrimaryKey, IntoSorterFn, IntoFilterFn};
use crate::state::{optional_rc_ptr_eq, DataStore, DataNode, DataNodeDerefGuard};

/// Hook to use a [TreeStore] with functional components.
///
/// This hook returns a [TreeStore] that listens to [TreeStore] change
/// events which trigger a redraw.
#[hook]
pub fn use_tree_store<F: FnOnce() -> TreeStore<T>, T: 'static>(init_fn: F) -> TreeStore<T> {

    let redraw = use_state(|| 0);

    let tree = use_state(init_fn);
    let _on_change = use_state({
        let tree = tree.clone();
        let redraw = redraw.clone();
        move || (*tree).add_listener(move |()| redraw.set(0)) // trigger redraw
    });

    (*tree).clone()
}

/// Owns the  listener callback. When dropped, the
/// listener callback will be removed from the [TreeStore].
pub struct TreeStoreObserver<T> {
    key: usize,
    inner: Rc<RefCell<KeyedSlabTree<T>>>,
}

impl<T> Drop for TreeStoreObserver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

/// Shared tree store (wrapper for [KeyedSlabTree]).
///
/// Functional components can use the [use_tree_store] hook.
///
/// # Note
///
/// A [TreeStore] is a shared state behind `Rc<RefCell<state>>`, so
/// a simply `PartialEq` would always return true. Please register a
/// listener to get notified about changes.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct TreeStore<T: 'static> {
    // Allow to store one TreeStoreObserver here (for convenience)
    #[derivative(PartialEq(compare_with="optional_rc_ptr_eq"))]
    on_change: Option<Rc<TreeStoreObserver<T>>>,
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    inner: Rc<RefCell<KeyedSlabTree<T>>>,
}

impl<T: ExtractPrimaryKey + 'static> TreeStore<T> {

    /// Creates a new instance for types implementing [ExtractPrimaryKey].
    ///
    /// Use [Self::with_extract_key] for types which does not
    /// implement [ExtractPrimaryKey].
    pub fn new() -> Self {
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(KeyedSlabTree::new())),
        }
    }
}

impl<T: 'static> TreeStore<T> {

    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        let tree = KeyedSlabTree::with_extract_key(extract_key);
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(tree)),
        }
    }

    /// Builder style method to set the 'view_root' flag.
    pub fn view_root(self, view_root: bool) -> Self {
        self.write().set_view_root(view_root);
        self
    }
    /// Builder style method to set the on_change callback.
    ///
    /// This calls [Self::add_listener] to create a new
    /// [TreeStoreObserver]. The observer is stored inside the
    /// [TreeStore] object, so each clone can hold a single on_select
    /// callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_change = match cb.into_event_callback() {
            Some(cb) => Some(Rc::new(self.add_listener(cb))),
            None => None,
        };
        self
    }

    /// Lock this store for read access.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably locked.
    pub fn read(&self) -> TreeStoreReadGuard<T> {
        let tree = self.inner.borrow();
        TreeStoreReadGuard { tree }
    }

    /// Lock this store for write access.
    ///
    /// # Panics
    ///
    /// Panics if the store is already locked.
    pub fn write(&self) -> TreeStoreWriteGuard<T> {
        let tree = self.inner.borrow_mut();

        TreeStoreWriteGuard {
            initial_version: tree.version(),
            tree,
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
    /// observer inside the [TreeStore] object.
    pub fn add_listener(&self, cb: impl Into<Callback<()>>) -> TreeStoreObserver<T> {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        TreeStoreObserver { key, inner: self.inner.clone() }
    }

    /// Set the sorter function.
    pub fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        self.write().set_sorter(sorter);
    }

    /// Set the filter function.
    pub fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        self.write().set_filter(filter);
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let mut tree = self.inner.borrow_mut();
        tree.update_filtered_data();
        tree.lookup_filtered_record_key(cursor)
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        let mut tree = self.inner.borrow_mut();
        tree.update_filtered_data();
        tree.filtered_record_pos(key)
    }

    fn filtered_data_len(&self) -> usize {
        let mut tree = self.inner.borrow_mut();
        tree.update_filtered_data();
        tree.filtered_data_len()
    }
}

impl<T: 'static> DataStore for TreeStore<T> {
    type Observer = TreeStoreObserver<T>;
    type Record = T;

    fn extract_key(&self, data: &T) -> Key {
        self.extract_key(data)
    }

    fn add_listener(&self, cb: impl Into<Callback<()>>) -> TreeStoreObserver<T> {
        self.add_listener(cb)
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

    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.inner.borrow_mut().update_filtered_data();
        Box::new(TreeStoreIterator {
            range: None,
            pos: 0,
            tree: self.inner.borrow(),
        })
    }

    fn filtered_data_range<'a>(
        &'a self,
        range: Range<usize>,
    ) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
        self.inner.borrow_mut().update_filtered_data();
        Box::new(TreeStoreIterator {
            pos: range.start,
            range: Some(range),
            tree: self.inner.borrow(),
        })
    }
}


/// Wraps a borrowed reference to a [TreeStore]
pub struct TreeStoreReadGuard<'a, T> {
    tree: Ref<'a, KeyedSlabTree<T>>,
}

impl<T> Deref for TreeStoreReadGuard<'_, T> {
    type Target = KeyedSlabTree<T>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

/// A wrapper type for a mutably borrowed [TreeStore]
pub struct TreeStoreWriteGuard<'a, T> {
    tree: RefMut<'a, KeyedSlabTree<T>>,
    initial_version: usize,
}

impl<T> Deref for TreeStoreWriteGuard<'_, T> {
    type Target = KeyedSlabTree<T>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl<'a, T> DerefMut for TreeStoreWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl<'a, T> Drop for TreeStoreWriteGuard<'a, T> {
    fn drop(&mut self) {
        if self.tree.version() != self.initial_version {
            self.tree.notify_listeners();
        }
    }
}

#[doc(hidden)]
pub struct KeyedSlabTreeBorrowRef<'a, T: 'static> {
    node_id: usize,
    tree: Ref<'a, KeyedSlabTree<T>>,
}

impl<'a, T> DataNode<T> for KeyedSlabTreeBorrowRef<'a, T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let guard = Box::new(RecordGuard {
            node_id: self.node_id,
            tree: Ref::clone(&self.tree),
        });
        DataNodeDerefGuard { guard }
    }
    fn level(&self) -> usize {
        let level = self.tree.get(self.node_id).unwrap().level;
        if !self.tree.view_root {
            level.saturating_sub(1)
        } else  {
            level
        }
    }
    fn expanded(&self) -> bool {
        self.tree.get(self.node_id).unwrap().expanded
    }
    fn is_leaf(&self) -> bool {
        self.tree.get(self.node_id).unwrap().children.is_none()
    }
    fn is_root(&self) -> bool {
        self.tree.tree.root_id == Some(self.node_id)
    }
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> {
       let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let parent_id = match entry.parent_id {
            Some(parent_id) => parent_id,
            None => return None,
        };

        let parent = Box::new(KeyedSlabTreeBorrowRef {
            node_id: parent_id,
            tree: Ref::clone(&self.tree),
        });

        Some(parent)
    }
    fn key(&self) -> Key {
        let record = &self.tree.get(self.node_id).unwrap().record;
        self.tree.extract_key(record)
    }
}

pub struct RecordGuard<'a, T> {
    node_id: usize,
    tree: Ref<'a, KeyedSlabTree<T>>,
}

pub struct RecordGuardMut<'a, T> {
    node_id: usize,
    tree: RefMut<'a, KeyedSlabTree<T>>,
}

impl<T> Deref for RecordGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let entry = self.tree.get(self.node_id).unwrap();
        &entry.record
    }
}

impl<T> Deref for RecordGuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let entry = self.tree.get(self.node_id).unwrap();
        &entry.record
    }
}

impl<T> DerefMut for RecordGuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        let entry = self.tree.get_mut(self.node_id).unwrap();
        &mut entry.record
    }
}

#[doc(hidden)]
pub struct TreeStoreIterator<'a, T: 'static> {
    tree: Ref<'a, KeyedSlabTree<T>>,
    pos: usize,
    range: Option<Range<usize>>,
}

impl <'a, T: 'static> Iterator for TreeStoreIterator<'a, T> where Self: 'a {
    type Item = (usize, Box<dyn DataNode<T> + 'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = &self.range {
            if range.end <= self.pos {
                return None;
            }
        }

        if self.tree.linear_view.len() <= self.pos {
            return None;
        }

        let pos = self.pos;
        self.pos += 1;

        let node_id = self.tree.linear_view[pos];
        let node = Box::new(KeyedSlabTreeBorrowRef {
            node_id,
            tree: Ref::clone(&self.tree),
        });

        Some((pos, node))
    }
}
