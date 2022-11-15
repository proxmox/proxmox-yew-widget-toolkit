use std::cmp::Ordering;

use slab::Slab;

use yew::virtual_dom::Key;
use yew::Callback;
use crate::props::{ExtractKeyFn, ExtractPrimaryKey, IntoFilterFn, IntoSorterFn, SorterFn, FilterFn};

use super::{SlabTree, SlabTreeEntry};

/// Tree implementation with filter and sorting functionality.
///
/// Based on [SlabTree], with added feature:
///
/// - Filtering
/// - Sorting
/// - Cursor
/// - Listener callbacks
pub struct KeyedSlabTree<T> {
    pub(crate) tree: SlabTree<T>,
    pub(crate) linear_view: Vec<usize>, // node_id list
    last_view_version: usize,

    pub(crate) extract_key: ExtractKeyFn<T>,

    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,
    cursor: Option<usize>,

    listeners: Slab<Callback<()>>,

    pub(crate) view_root: bool,
}

/// An immutable reference to a [KeyedSlabTree] node.
pub struct KeyedSlabTreeNodeRef<'a, T> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a KeyedSlabTree<T>,
}

impl<'a, T> KeyedSlabTreeNodeRef<'a, T> {
    fn new(tree: &'a KeyedSlabTree<T>, node_id: usize) -> Self {
        Self { node_id, tree }
    }
}

/// A mutable reference to a [KeyedSlabTree] node.
pub struct KeyedSlabTreeNodeMut<'a, T> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a mut KeyedSlabTree<T>,
}

impl<'a, T> KeyedSlabTreeNodeMut<'a, T> {
    fn new(tree: &'a mut KeyedSlabTree<T>, node_id: usize) -> Self {
        Self { node_id, tree }
    }
}

impl<'a, T> KeyedSlabTreeNodeMut<'a, T> {
    impl_slab_node_ref!(KeyedSlabTreeNodeRef<T>);
    impl_slab_node_mut!(KeyedSlabTreeNodeMut<T>, KeyedSlabTree<T>);

    /// Iterate over children.
    pub fn children(&self) -> KeyedSlabTreeChildren<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        KeyedSlabTreeChildren {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    /// Iterate over children (mutable).
    pub fn children_mut(&mut self) -> KeyedSlabTreeChildrenMut<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        KeyedSlabTreeChildrenMut {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    /// Visit a subtree in pre-order (mutable)
    pub fn visit(&self, visitor: &mut impl FnMut(&KeyedSlabTreeNodeRef<T>)) {
        let node_ref = KeyedSlabTreeNodeRef::new(self.tree, self.node_id);
        visitor(&node_ref);
        self.visit_children(visitor);
    }

    /// Retunrs the unique record key.
    pub fn key(&self) -> Key {
        self.tree.extract_key(self.record())
    }

    /// Find a node by key.
    pub fn find_node_by_key(&self, key: &Key) -> Option<KeyedSlabTreeNodeRef<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| KeyedSlabTreeNodeRef { node_id, tree: self.tree })
    }

    /// Find a node by key (mutable).
    pub fn find_node_by_key_mut(&mut self, key: &Key) -> Option<KeyedSlabTreeNodeMut<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| KeyedSlabTreeNodeMut { node_id, tree: self.tree })
    }

    /// Remove a descendent node by key.
    ///
    /// # Note
    ///
    /// This cannot be used to remove `self` from the tree.
    ///
    pub fn remove_descendent_by_key(&mut self, key: &Key) -> Option<T> {
        if let Some(pos) = self.children().position(|child| &child.key() == key) {
            return self.remove_child(pos);
        }

        for mut child in self.children_mut() {
            let result = child.remove_descendent_by_key(key);
            if result.is_some() {
                return result;
            }
        }

        None
    }
}

impl<'a, T> KeyedSlabTreeNodeRef<'a, T> {
    impl_slab_node_ref!(KeyedSlabTreeNodeRef<T>);

    /// Iterate over children.
    pub fn children(&self) -> KeyedSlabTreeChildren<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        KeyedSlabTreeChildren {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    /// Visit a subtree in pre-order
    pub fn visit(&self, visitor: &mut impl FnMut(&KeyedSlabTreeNodeRef<T>)) {
        visitor(self);
        self.visit_children(visitor);
    }

    /// Retunrs the unique record key.
    pub fn key(&self) -> Key {
        self.tree.extract_key(self.record())
    }

    /// Find a node by key.
    pub fn find_node_by_key(&self, key: &Key) -> Option<KeyedSlabTreeNodeRef<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| KeyedSlabTreeNodeRef { node_id, tree: self.tree })
    }
}

impl<T> From<KeyedSlabTree<T>> for SlabTree<T> {
    fn from(tree: KeyedSlabTree<T>) -> Self {
        tree.tree
    }
}

impl<T: ExtractPrimaryKey> KeyedSlabTree<T> {

    pub fn new() -> Self {
        let extract_key = ExtractKeyFn::new(|data: &T| data.extract_key());
        Self::with_extract_key(extract_key)
    }
}

impl<T> KeyedSlabTree<T> {

    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            extract_key: extract_key.into(),
            tree: SlabTree::new(),
            linear_view: Vec::new(),
            last_view_version: 0,
            sorter: None,
            filter: None,
            cursor: None,
            listeners: Slab::new(),
            view_root: true,
         }
    }

    /// Set flag to show/hide the root node.
    pub fn set_view_root(&mut self, view_root: bool) {
        self.record_data_change();
        self.view_root = view_root;
    }

    /// Tree version number (incread by any modification).
    pub fn version(&self) -> usize {
        self.tree.version()
    }

    pub(crate) fn record_data_change(&mut self) {
        self.tree.record_data_change();
    }

    /// Returns the unique record key.
    pub fn extract_key(&self, data: &T) -> Key {
        self.extract_key.apply(data)
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

    pub(crate) fn set_sorter(&mut self, sorter: impl IntoSorterFn<T>) {
        self.sorter = sorter.into_sorter_fn();
        self.tree.record_data_change();
    }

    pub(crate) fn set_filter(&mut self, filter: impl IntoFilterFn<T>) {
        self.filter = filter.into_filter_fn();
        self.tree.record_data_change();
    }

    fn flatten_tree_children(
        &self,
        list: &mut Vec<usize>,
        children: &[usize],
    ) {
        let mut children: Vec<usize> = children.iter()
            .filter(|child_id| {
                let child_id = **child_id;
                let entry = self.get(child_id).unwrap();
                match &self.filter {
                    Some(filter) => filter.apply(0, &entry.record),
                    None => true,
                }
            })
            .map(|child_id| *child_id)
            .collect();

        if let Some(sorter) = &self.sorter {
            children.sort_by(|child_id_a, child_id_b| {
                let entry_a = self.get(*child_id_a).unwrap();
                let entry_b = self.get(*child_id_b).unwrap();
                sorter.cmp(&entry_a.record, &entry_b.record)
            });
        }

        for child_id in children.into_iter() {
            list.push(child_id);
            let entry = self.get(child_id).unwrap();
            if entry.expanded {
                if let Some(children) = &entry.children {
                    self.flatten_tree_children(list, children);
                }
            }
        }
    }

    pub(crate) fn update_filtered_data(&mut self) {
        if self.tree.version() == self.last_view_version {
            return;
        }

        let mut view = Vec::new();

        if let Some(root_id) = self.tree.root_id {
            let root = self.get(root_id).unwrap();
            if self.view_root { view.push(root_id); }
            if !self.view_root || root.expanded {
                if let Some(children) = &root.children {
                    self.flatten_tree_children(&mut view, children);
                }
            }
        }

        self.linear_view = view;
        self.last_view_version = self.tree.version();
    }

    pub(crate) fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let node_id = match self.linear_view.get(cursor) {
            Some(node_id) => *node_id,
            None => return None,
        };

        let entry = match self.get(node_id) {
            Some(entry) => entry,
            None => return None,
        };

        Some(self.extract_key(&entry.record))
    }

    pub(crate) fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.linear_view.iter()
            .position(|node_id| {
                let entry = self.get(*node_id).unwrap();
                key == &self.extract_key(&entry.record)
            })
    }

    pub(crate) fn filtered_data_len(&self) -> usize {
        self.linear_view.len()
    }

    pub(crate) fn get_cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub(crate) fn set_cursor(&mut self, cursor: Option<usize>) {
        self.cursor = match cursor {
            Some(c) => {
                let len = self.filtered_data_len();
                if c < len {
                    Some(c)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Set the root node.
    ///
    /// The current tree (if any) is discarded.
    pub fn set_root(&mut self, record: T) -> KeyedSlabTreeNodeMut<T> {
        let node = self.tree.set_root(record);
        KeyedSlabTreeNodeMut {
            node_id: node.node_id,
            tree: self,
        }
    }

    /// Set the whole tree
    ///
    /// The current tree (if any) is discarded.
    pub fn set_root_tree(&mut self, data: impl Into<SlabTree<T>>) {
        self.tree.set_root_tree(data);
    }

    pub(crate) fn get(&self, node_id: usize) -> Option<&SlabTreeEntry<T>> {
        self.tree.get(node_id)
    }

    pub(crate) fn get_mut(&mut self, node_id: usize) -> Option<&mut SlabTreeEntry<T>> {
        self.tree.get_mut(node_id)
    }

    /// Returns the root node.
    pub fn root(&self) -> Option<KeyedSlabTreeNodeRef<T>> {
        self.tree.root_id.map(|root_id| KeyedSlabTreeNodeRef {
            node_id: root_id,
            tree: self,
        })
    }

    /// Returns the mutable root node.
    pub fn root_mut(&mut self) -> Option<KeyedSlabTreeNodeMut<T>> {
        self.tree.root_id.map(|root_id| KeyedSlabTreeNodeMut {
            node_id: root_id,
            tree: self,
        })
    }

    fn append_subtree_node(&mut self, subtree: &mut SlabTree<T>, subtree_node: usize, level: usize, parent: usize) -> usize {
        self.tree.append_subtree_node(subtree, subtree_node, level, parent)
    }

    fn remove_node_id(&mut self, node_id: usize) -> Option<T> {
        self.tree.remove_node_id(node_id)
    }

    fn remove_tree_node_id(&mut self, node_id: usize) -> Option<KeyedSlabTree<T>> {
        match self.tree.remove_tree_node_id(node_id) {
            Some(tree) => {
                Some(KeyedSlabTree {
                    extract_key: self.extract_key.clone(),
                    tree,
                    linear_view: Vec::new(),
                    last_view_version: 0,
                    sorter: None,
                    filter: None,
                    cursor: None,
                    listeners: Slab::new(),
                    view_root: true,
                })
            }
            None => None,
        }
    }

    fn insert_record(&mut self, record: T, parent_id: Option<usize>) -> usize  {
        self.tree.insert_record(record, parent_id)
    }

    fn find_subnode_by_key(&self, node_id: usize, key: &Key) -> Option<usize> {
        let entry = match self.get(node_id) {
            Some(entry) => entry,
            None => return None,
        };

        if key == &self.extract_key(&entry.record) {
            return Some(node_id);
        }

        if let Some(children) = &entry.children {
            for child_id in children {
                if let Some(pos) = self.find_subnode_by_key(*child_id, key) {
                    return Some(pos);
                }
            }
        }

        None
    }

    fn find_node_by_key(&self, key: &Key) -> Option<usize> {
        match self.tree.root_id {
            None => None,
            Some(root_id) => self.find_subnode_by_key(root_id, key),
        }
    }

    pub(crate) fn sort_node<F>(&mut self, recursive: bool, node_id: usize, compare: &mut F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.tree.sort_node(recursive, node_id, compare);
    }

    /// Sort the tree node recursively
    pub fn sort(&mut self, recursive: bool)
    where
        T: Ord,
    {
        self.tree.sort_by(recursive, &mut T::cmp);
    }

    /// Sort the tree recursively
    pub fn sort_by<F>(&mut self, recursive: bool, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.tree.sort_by(recursive, compare);
    }

    /// Find a node by its key.
    pub fn lookup_node(&self, key: &Key) -> Option<KeyedSlabTreeNodeRef<T>> {
        self.find_node_by_key(key)
            .map(|node_id| {
                KeyedSlabTreeNodeRef {
                    node_id: node_id,
                    tree: self,
                }
            })
    }

    /// Find a node by its key (mutable).
    pub fn lookup_node_mut(&mut self, key: &Key) -> Option<KeyedSlabTreeNodeMut<T>> {
        self.find_node_by_key(key)
            .map(|node_id| {
                KeyedSlabTreeNodeMut {
                    node_id: node_id,
                    tree: self,
                }
            })
    }

}

/// [KeyedSlabTree] iterator over a node`s children.
pub struct KeyedSlabTreeChildren<'a, T> {
    pos: Option<usize>,
    node_id: usize,
    tree: &'a KeyedSlabTree<T>,
}

impl_slab_tree_child_iter!(KeyedSlabTreeChildren, KeyedSlabTreeNodeRef);

/// [KeyedSlabTree] iterator over a node`s children (mutable).
pub struct KeyedSlabTreeChildrenMut<'a, T> {
    pos: Option<usize>,
    node_id: usize,
    tree: &'a mut KeyedSlabTree<T>,
}

impl_slab_tree_child_iter_mut!(KeyedSlabTreeChildrenMut, KeyedSlabTreeNodeMut);
