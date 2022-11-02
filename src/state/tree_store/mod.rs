mod slab_tree;
pub use slab_tree::{SlabTree, SlabTreeNodeMut};

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
    inner: Rc<RefCell<SlabTree<T>>>,
}

impl<T> Drop for TreeStoreObserver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().remove_listener(self.key);
    }
}

/// Shared tree store.
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
    inner: Rc<RefCell<SlabTree<T>>>,
}

impl<T: ExtractPrimaryKey + 'static> TreeStore<T> {

    /// Creates a new instance for types implementing [ExtractPrimaryKey].
    ///
    /// Use [Self::with_extract_key] for types which does not
    /// implement [ExtractPrimaryKey].
    pub fn new() -> Self {
        let extract_key = ExtractKeyFn::new(|data: &T| data.extract_key());
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(SlabTree::new(extract_key))),
         }
    }
}

impl<T: 'static> TreeStore<T> {

    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            on_change: None,
            inner: Rc::new(RefCell::new(SlabTree::new(extract_key.into()))),
        }
    }

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
        TreeStoreReadGuard {
            tree: self.inner.borrow(),
        }
    }

    /// Lock this store for write access.
    ///
    /// # Panics
    ///
    /// Panics if the store is already locked.
    pub fn write(&self) -> TreeStoreWriteGuard<T> {
        let tree = self.inner.borrow_mut();

        TreeStoreWriteGuard {
            initial_version: tree.version,
            tree,
        }
    }

    pub fn root_mut(&mut self) -> Option<SlabTreeNodeShared<T>> {
        let mut tree = self.inner.borrow_mut();
        match tree.root_id() {
            None => None,
            Some(root_id) => Some(SlabTreeNodeShared {
                node_id: root_id,
                inner: self.inner.clone(),
            }),
        }
    }

    pub fn set_root(&self, record: T) -> SlabTreeNodeShared<T> {
        let mut tree = self.inner.borrow_mut();
        let node = tree.set_root(record);

        let node = SlabTreeNodeShared {
            node_id: node.node_id(),
            inner: self.inner.clone(),
        };

        tree.notify_listeners();

        node
    }

    pub fn get_expanded(&self, key: &Key) -> bool {
        self.inner.borrow().get_expanded_key(key)
    }

    pub fn set_expanded(&self, key: &Key, expanded: bool) {
        let mut tree = self.inner.borrow_mut();
        tree.set_expanded_key(key, expanded);
        tree.notify_listeners();
    }

    pub fn toggle_expanded(&self, key: &Key) {
        let mut tree = self.inner.borrow_mut();
        tree.toggle_expanded_key(key);
        tree.notify_listeners();
    }
}

impl<T> DataStore<T> for TreeStore<T> {
    type Observer = TreeStoreObserver<T>;

    fn extract_key(&self, data: &T) -> Key {
        self.inner.borrow().extract_key.apply(data)
    }

    fn add_listener(&self, cb: impl Into<Callback<()>>) -> TreeStoreObserver<T> {
        let key = self.inner.borrow_mut()
            .add_listener(cb.into());
        TreeStoreObserver { key, inner: self.inner.clone() }
    }

    fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        let mut tree = self.inner.borrow_mut();
        tree.set_sorter(sorter);
        tree.notify_listeners();
    }

    fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        let mut tree = self.inner.borrow_mut();
        tree.set_filter(filter);
        tree.notify_listeners();
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

    fn get_cursor(&self) -> Option<usize> {
        self.inner.borrow().get_cursor()
    }

    fn set_cursor(&self, cursor: Option<usize>) {
        let mut tree = self.inner.borrow_mut();
        tree.update_filtered_data();
        tree.set_cursor(cursor);
        tree.notify_listeners();
    }
}



pub struct TreeStoreWriteGuard<'a, T> {
    tree: RefMut<'a, SlabTree<T>>,
    initial_version: usize,
}

impl<T> Deref for TreeStoreWriteGuard<'_, T> {
    type Target = SlabTree<T>;

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
        if self.tree.version != self.initial_version {
            self.tree.notify_listeners();
        }
    }
}

pub struct TreeStoreReadGuard<'a, T> {
    tree: Ref<'a, SlabTree<T>>,
}

impl<T> Deref for TreeStoreReadGuard<'_, T> {
    type Target = SlabTree<T>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

pub struct SlabTreeNodeRef<'a, T: 'static> {
    node_id: usize,
    tree: Ref<'a, SlabTree<T>>,
}

impl<'a, T> DataNode<T> for SlabTreeNodeRef<'a, T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let guard = Box::new(RecordGuard {
            node_id: self.node_id,
            tree: Ref::clone(&self.tree),
        });
        DataNodeDerefGuard { guard }
    }
    fn level(&self) -> usize {
        self.tree.get(self.node_id).unwrap().level
    }
    fn expanded(&self) -> bool {
        self.tree.get(self.node_id).unwrap().expanded
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

        let parent = Box::new(SlabTreeNodeRef {
            node_id: parent_id,
            tree: Ref::clone(&self.tree),
        });

        Some(parent)
    }

}

pub struct SlabTreeNodeShared<T> {
    node_id: usize,
    inner: Rc<RefCell<SlabTree<T>>>,
}

pub struct RecordGuard<'a, T> {
    node_id: usize,
    tree: Ref<'a, SlabTree<T>>,
}

pub struct RecordGuardMut<'a, T> {
    node_id: usize,
    tree: RefMut<'a, SlabTree<T>>,
}

impl<T> DataNode<T> for SlabTreeNodeShared<T> {
    fn record(&self) -> DataNodeDerefGuard<T> {
        let guard: Box<RecordGuard<T>> = Box::new(SlabTreeNodeShared::record(self));
        DataNodeDerefGuard { guard }
    }
    fn level(&self) -> usize {
        self.inner.borrow().get(self.node_id).unwrap().level
    }
    fn expanded(&self) -> bool {
        self.inner.borrow().get(self.node_id).unwrap().expanded
    }
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>> {
        let tree = self.inner.borrow();
        let entry = match tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let parent_id = match entry.parent_id {
            Some(parent_id) => parent_id,
            None => return None,
        };

        let parent = Box::new(SlabTreeNodeShared {
            node_id: parent_id,
            inner: Rc::clone(&self.inner),
        });

        Some(parent)
    }
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

impl<T> SlabTreeNodeShared<T> {

    /// Mutable reference to the node data.
    pub fn record_mut(&self) -> RecordGuardMut<T> {
        RecordGuardMut {
            node_id: self.node_id,
            tree: self.inner.borrow_mut(),
        }
    }

    pub fn record(&self) -> RecordGuard<T> {
        RecordGuard {
            node_id: self.node_id,
            tree: self.inner.borrow(),
        }
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.inner.borrow_mut().get_node_ref_mut(self.node_id)
            .set_expanded(expanded);
    }

    /// Appends a new node as the last child. Returns a [SlabTreeNodeShared] to the newly added node.
    pub fn append(&mut self, record: T) -> SlabTreeNodeShared<T> {
        let mut tree = self.inner.borrow_mut();
        let child_id = tree.insert_entry(record, Some(self.node_id));

        let entry = tree.get_mut(self.node_id).unwrap();
        if let Some(children) = &mut entry.children {
            children.push(child_id);
        } else {
            entry.children = Some(vec![child_id]);
        }

        SlabTreeNodeShared {
            node_id: child_id,
            inner: self.inner.clone(),
        }
    }

}

pub struct TreeStoreIterator<'a, T: 'static> {
    tree: Ref<'a, SlabTree<T>>,
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
        let node = Box::new(SlabTreeNodeRef {
            node_id,
            tree: Ref::clone(&self.tree),
        });

        Some((pos, node))
    }
}
