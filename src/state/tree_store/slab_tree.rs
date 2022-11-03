use slab::Slab;

use yew::virtual_dom::Key;
use yew::Callback;
use crate::props::{ExtractKeyFn, IntoFilterFn, IntoSorterFn, SorterFn, FilterFn};

pub(crate) struct SlabTreeEntry<T> {
    //pub(crate) node_id: usize,
    pub(crate) parent_id: Option<usize>,
    pub(crate) level: usize,
    pub(crate) record: T,
    pub(crate) expanded: bool,
    pub(crate) children: Option<Vec<usize>>,
}

/// Tree implementation backup by a vector ([Slab])
pub struct SlabTree<T> {
    root_id: Option<usize>,

    tree: Slab<SlabTreeEntry<T>>,

    pub(crate) version: usize, // for change tracking
    pub(crate) linear_view: Vec<usize>, // node_id list
    last_view_version: usize,

    pub(crate) extract_key: ExtractKeyFn<T>,

    sorter: Option<SorterFn<T>>,
    filter: Option<FilterFn<T>>,
    cursor: Option<usize>,

    listeners: Slab<Callback<()>>,
}

/// A mutable reference to a [SlabTree] node.
pub struct SlabTreeNodeMut<'a, T> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a mut SlabTree<T>,
}

impl<'a, T> SlabTreeNodeMut<'a, T> {

    /// Appends a new node as the last child. Returns a [SlabTreeNodeMut] to the newly added node.
    pub fn append(&mut self, record: T) -> SlabTreeNodeMut<T> {

        let child_id = self.tree.insert_entry(record, Some(self.node_id));

        let entry = self.tree.tree.get_mut(self.node_id).unwrap();
        if let Some(children) = &mut entry.children {
            children.push(child_id);
        } else {
            entry.children = Some(vec![child_id]);
        }

        SlabTreeNodeMut {
             node_id: child_id,
             tree: self.tree,
        }
    }

    // /// Returns the unique node id.
    // pub fn node_id(&self) -> usize {
    //     self.node_id
    // }

    /// Retunrs the unique record key.
    pub fn key(&self) -> Key {
        self.tree.extract_key(self.record())
    }

    /// Node nesting level
    pub fn level(&self) -> usize {
        self.tree.get(self.node_id).unwrap().level
    }

    /// Get the expanded flag
    pub fn expanded(&self) -> bool {
        self.tree.tree.get(self.node_id).unwrap().expanded
    }

    /// Set the expanded flag
    pub fn set_expanded(&mut self, expanded: bool) {
        if self.expanded() != expanded {
            self.tree.record_data_change();
            let entry = self.tree.tree.get_mut(self.node_id).unwrap();
            entry.expanded = expanded;
        }
    }

    /// Reference to the data record.
    pub fn record(&self) -> &T {
        &self.tree.get(self.node_id).unwrap().record
    }

    /// Mutable reference to the data record.
    pub fn record_mut(&mut self) -> &mut T {
        self.tree.record_data_change();
        let entry = self.tree.tree.get_mut(self.node_id).unwrap();
        &mut entry.record
    }

    /// Sort the tree node recursively
    pub fn sort(&mut self, sorter: &SorterFn<T>) {
        self.tree.sort_node(self.node_id, sorter);
    }

    /// Get the parent node.
    pub fn parent(&self) -> Option<SlabTreeNodeRef<T>> {
        let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let parent_id = match entry.parent_id {
            Some(parent_id) => parent_id,
            None => return None,
        };

        Some(SlabTreeNodeRef {
            node_id: parent_id,
            tree: self.tree,
        })
    }

    /// Get a mutable ref to the parent node.
    pub fn parent_mut(&mut self) -> Option<SlabTreeNodeMut<T>> {
        let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let parent_id = match entry.parent_id {
            Some(parent_id) => parent_id,
            None => return None,
        };

        Some(SlabTreeNodeMut {
            node_id: parent_id,
            tree: self.tree,
        })
    }

    /// Find a node by key.
    pub fn find_node_by_key(&self, key: &Key) -> Option<SlabTreeNodeRef<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| SlabTreeNodeRef { node_id, tree: self.tree })
    }

    /// Find a node by key (mutable).
    pub fn find_node_by_key_mut(&mut self, key: &Key) -> Option<SlabTreeNodeMut<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| SlabTreeNodeMut { node_id, tree: self.tree })
    }

}

/// An immutable reference to a [SlabTree] node.
pub struct SlabTreeNodeRef<'a, T: 'static> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a SlabTree<T>,
}

impl<'a, T: 'static> SlabTreeNodeRef<'a, T> {

    /// Reference to the data record.
    pub fn record(&self) -> &T {
        &self.tree.get(self.node_id).unwrap().record
    }

    // /// Returns the unique node id.
    //pub fn node_id(&self) -> usize {
    //    self.node_id
    //}

    /// Retunrs the unique record key.
    pub fn key(&self) -> Key {
        self.tree.extract_key(self.record())
    }

    /// Node nesting level
    pub fn level(&self) -> usize {
        self.tree.get(self.node_id).unwrap().level
    }

    /// Get the node entry
    pub(crate) fn get(&self) -> &SlabTreeEntry<T> {
        self.tree.get(self.node_id).unwrap()
    }

    /// Get the expanded flag
    pub fn expanded(&self) -> bool {
        self.tree.get(self.node_id).unwrap().expanded
    }

    /// Get the parent node.
    pub fn parent(&self) -> Option<SlabTreeNodeRef<T>> {
        let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let parent_id = match entry.parent_id {
            Some(parent_id) => parent_id,
            None => return None,
        };

        Some(SlabTreeNodeRef {
            node_id: parent_id,
            tree: self.tree,
        })
    }

    /// Find a node by key.
    pub fn find_node_by_key(&self, key: &Key) -> Option<SlabTreeNodeRef<T>> {
        self.tree.find_subnode_by_key(self.node_id, key)
            .map(|node_id| SlabTreeNodeRef { node_id, tree: self.tree })
    }
}

impl<T> SlabTree<T> {

    pub fn new(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        let tree = Slab::new();
        Self {
            extract_key: extract_key.into(),
            root_id: None,
            tree,
            version: 0,
            linear_view: Vec::new(),
            last_view_version: 0,
            sorter: None,
            filter: None,
            cursor: None,
            listeners: Slab::new(),
         }
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
        self.record_data_change();
    }

    pub(crate) fn set_filter(&mut self, filter: impl IntoFilterFn<T>) {
        self.filter = filter.into_filter_fn();
        self.record_data_change();
    }

    fn flatten_tree_children(
        &self,
        list: &mut Vec<usize>,
        children: &[usize],
    ) {
        let mut children: Vec<usize> = children.iter()
            .filter(|child_id| {
                let child_id = **child_id;
                let entry = self.tree.get(child_id).unwrap();
                match &self.filter {
                    Some(filter) => filter.apply(0, &entry.record),
                    None => true,
                }
            })
            .map(|child_id| *child_id)
            .collect();

        if let Some(sorter) = &self.sorter {
            children.sort_by(|child_id_a, child_id_b| {
                let entry_a = self.tree.get(*child_id_a).unwrap();
                let entry_b = self.tree.get(*child_id_b).unwrap();
                sorter.cmp(&entry_a.record, &entry_b.record)
            });
        }

        for child_id in children.into_iter() {
            list.push(child_id);
            let entry = self.tree.get(child_id).unwrap();
            if entry.expanded {
                if let Some(children) = &entry.children {
                    self.flatten_tree_children(list, children);
                }
            }
        }
    }

    pub(crate) fn update_filtered_data(&mut self) {
        if self.version == self.last_view_version {
            return;
        }

        let mut view = Vec::new();

        if let Some(root_id) = self.root_id {
            let root = self.tree.get(root_id).unwrap();
            view.push(root_id);
            if root.expanded {
                if let Some(children) = &root.children {
                    self.flatten_tree_children(&mut view, children);
                }
            }
        }

        self.linear_view = view;
        self.last_view_version = self.version;
    }

    pub(crate) fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        let node_id = match self.linear_view.get(cursor) {
            Some(node_id) => *node_id,
            None => return None,
        };

        let entry = match self.tree.get(node_id) {
            Some(entry) => entry,
            None => return None,
        };

        Some(self.extract_key(&entry.record))
    }

    pub(crate) fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.linear_view.iter()
            .position(|node_id| {
                let entry = self.tree.get(*node_id).unwrap();
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

    fn record_data_change(&mut self) {
        self.version += 1;
    }

    /// Set the root node.
    ///
    /// The current tree (if any) is discarded.
    pub fn set_root(&mut self, record: T) -> SlabTreeNodeMut<T> {
        self.record_data_change();

        let last_root_id = self.root_id;
        let root_id = self.insert_entry(record, None);
        self.root_id = Some(root_id);
        if let Some(last_root_id) = last_root_id {
            self.remove(last_root_id);
        }
        SlabTreeNodeMut {
            node_id: root_id,
            tree: self,
        }
    }

    pub(crate) fn get(&self, node_id: usize) -> Option<&SlabTreeEntry<T>> {
        self.tree.get(node_id)
    }

    pub(crate) fn get_mut(&mut self, node_id: usize) -> Option<&mut SlabTreeEntry<T>> {
        self.tree.get_mut(node_id)
    }

    /// Returns the root node.
    pub fn root(&self) -> Option<SlabTreeNodeRef<T>> {
        self.root_id.map(|root_id| SlabTreeNodeRef {
            node_id: root_id,
            tree: self,
        })
    }

    /// Returns the mutable root node.
    pub fn root_mut(&mut self) -> Option<SlabTreeNodeMut<T>> {
        self.root_id.map(|root_id| SlabTreeNodeMut {
            node_id: root_id,
            tree: self,
        })
    }

    pub(crate) fn remove(&mut self, node_id: usize) -> Option<T> {
        if let Some(entry) = self.tree.try_remove(node_id) {
            self.record_data_change();

            // remove fro parents's child list
            if let Some(parent_id) = entry.parent_id {
                let parent = self.tree.get_mut(parent_id).unwrap();
                if let Some(parent_children) = &mut parent.children {
                    parent_children.retain(|id| *id != node_id);
                }
            }

            if let Some(children) = entry.children {
                for child_id in children {
                    self.remove(child_id);
                }
            }

            if Some(node_id) == self.root_id {
                self.root_id = None;
            }

            Some(entry.record)
        } else {
            None
        }
    }

    fn find_subnode_by_key(&self, node_id: usize, key: &Key) -> Option<usize> {
        let entry = match self.tree.get(node_id) {
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
        match self.root_id {
            None => None,
            Some(root_id) => self.find_subnode_by_key(root_id, key),
        }
    }

    fn sort_children(&mut self, children: &mut [usize], sorter: &SorterFn<T>) {
        children.sort_by(|child_id_a, child_id_b| {
            let entry_a = self.tree.get(*child_id_a).unwrap();
            let entry_b = self.tree.get(*child_id_b).unwrap();
            sorter.cmp(&entry_a.record, &entry_b.record)
        });

        for child_id in children {
            self.sort_node(*child_id, sorter);
        }
    }

    fn sort_node(&mut self, node_id: usize, sorter: &SorterFn<T>) {
        self.record_data_change();
        let mut children = self.tree.get_mut(node_id).unwrap().children.take();
        if let Some(children) = &mut children {
            self.sort_children(children, sorter);
        }
        self.tree.get_mut(node_id).unwrap().children = children;
    }

    /// Sort the tree recursively
    pub fn sort(&mut self, sorter: &SorterFn<T>) {
        self.record_data_change();
        if let Some(root_id) = self.root_id {
            self.sort_node(root_id, sorter);
        }
    }

    pub(crate) fn insert_entry(&mut self, record: T, parent_id: Option<usize>) -> usize  {
        self.record_data_change();

        let level = if let Some(parent_id) = parent_id {
            self.tree.get(parent_id).unwrap().level + 1
        } else {
            0
        };

        let vacant_entry = self.tree.vacant_entry();
        let node_id =  vacant_entry.key();

        let entry = SlabTreeEntry {
            //node_id,
            parent_id,
            level,
            record,
            expanded: false,
            children: None,
        };
        vacant_entry.insert(entry);
        node_id
    }

    /// Find a node by its key.
    pub fn lookup_node(&self, key: &Key) -> Option<SlabTreeNodeRef<T>> {
        self.find_node_by_key(key)
            .map(|node_id| {
                SlabTreeNodeRef {
                    node_id: node_id,
                    tree: self,
                }
            })
    }

    /// Find a node by its key (mutable).
    pub fn lookup_node_mut(&mut self, key: &Key) -> Option<SlabTreeNodeMut<T>> {
        self.find_node_by_key(key)
            .map(|node_id| {
                SlabTreeNodeMut {
                    node_id: node_id,
                    tree: self,
                }
            })
    }

}
