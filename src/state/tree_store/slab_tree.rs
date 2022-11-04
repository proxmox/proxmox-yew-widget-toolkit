use slab::Slab;

use crate::props::SorterFn;

pub(crate) struct SlabTreeEntry<T> {
    pub(crate) parent_id: Option<usize>,
    pub(crate) level: usize,
    pub(crate) record: T,
    pub(crate) expanded: bool,
    pub(crate) children: Option<Vec<usize>>,
}

/// Tree implementation backup by a vector ([Slab])
pub struct SlabTree<T> {
    pub(crate) root_id: Option<usize>,
    pub(crate) slab: Slab<SlabTreeEntry<T>>,
    pub(crate) version: usize, // for change tracking
}

/// An immutable reference to a [SlabTree] node.
pub struct SlabTreeNodeRef<'a, T> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a SlabTree<T>,
}

impl<'a, T> SlabTreeNodeRef<'a, T> {
    fn new(tree: &'a SlabTree<T>, node_id: usize) -> Self {
        Self { node_id, tree }
    }
}

/// A mutable reference to a [SlabTree] node.
pub struct SlabTreeNodeMut<'a, T> {
    pub(crate) node_id: usize,
    pub(crate) tree: &'a mut SlabTree<T>,
}

impl<'a, T> SlabTreeNodeMut<'a, T> {
    fn new(tree: &'a mut SlabTree<T>, node_id: usize) -> Self {
        Self { node_id, tree }
    }
}

#[macro_export]
macro_rules! impl_slab_node_ref {
    ($R:ty) => {
        /// Reference to the data record.
        pub fn record(&self) -> &T {
            &self.tree.get(self.node_id).unwrap().record
        }

        /// Node nesting level
        pub fn level(&self) -> usize {
            self.tree.get(self.node_id).unwrap().level
        }

        /*
        /// Get the node entry
        pub(crate) fn get(&self) -> &SlabTreeEntry<T> {
            self.tree.get(self.node_id).unwrap()
        }
         */

        /// Get the expanded flag
        pub fn expanded(&self) -> bool {
            self.tree.get(self.node_id).unwrap().expanded
        }

        /// Get the parent node.
        pub fn parent(&self) -> Option<$R> {
            let entry = match self.tree.get(self.node_id) {
                Some(entry) => entry,
                None => return None,
            };

            let parent_id = match entry.parent_id {
                Some(parent_id) => parent_id,
                None => return None,
            };

            Some(<$R>::new(self.tree, parent_id))
        }
    }
}
#[macro_export]
macro_rules! impl_slab_node_mut {
    ($M:ty) => {
        /// Appends a new node as the last child. Returns a mutable ref to the newly added node.
        pub fn append(&mut self, record: T) -> $M {

            let child_id = self.tree.insert_entry(record, Some(self.node_id));

            let entry = self.tree.get_mut(self.node_id).unwrap();
            if let Some(children) = &mut entry.children {
                children.push(child_id);
            } else {
                entry.children = Some(vec![child_id]);
            }

            <$M>::new(self.tree, child_id)
        }

        /// Set the expanded flag
        pub fn set_expanded(&mut self, expanded: bool) {
            if self.expanded() != expanded {
                self.tree.record_data_change();
                let entry = self.tree.get_mut(self.node_id).unwrap();
                entry.expanded = expanded;
            }
        }

        /// Mutable reference to the data record.
        pub fn record_mut(&mut self) -> &mut T {
            self.tree.record_data_change();
            let entry = self.tree.get_mut(self.node_id).unwrap();
            &mut entry.record
        }

        /// Sort the tree node recursively
        pub fn sort(&mut self, sorter: &SorterFn<T>) {
            self.tree.sort_node(self.node_id, sorter);
        }

        /// Get a mutable ref to the parent node.
        pub fn parent_mut(&mut self) -> Option<$M> {
            let entry = match self.tree.get(self.node_id) {
                Some(entry) => entry,
                None => return None,
            };

            let parent_id = match entry.parent_id {
                Some(parent_id) => parent_id,
                None => return None,
            };

            Some(<$M>::new(self.tree, parent_id))
        }
    }
}

impl<'a, T> SlabTreeNodeMut<'a, T> {
    impl_slab_node_ref!{SlabTreeNodeRef<T>}
    impl_slab_node_mut!{SlabTreeNodeMut<T>}
}

impl<'a, T: 'static> SlabTreeNodeRef<'a, T> {
    impl_slab_node_ref!{SlabTreeNodeRef<T>}
}

impl<T> SlabTree<T> {

    pub fn new() -> Self {
        Self {
            root_id: None,
            slab: Slab::new(),
            version: 0,
        }
    }

    pub(crate) fn record_data_change(&mut self) {
        self.version += 1;
    }

    /// Tree version number (incread by any modification).
    pub fn version(&self) -> usize {
        self.version
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

    /// Set the whole tree
    ///
    /// The current tree (if any) is discarded.
    pub fn set_root_tree(&mut self, data: impl Into<SlabTree<T>>) {
        self.record_data_change();
        if let Some(last_root_id) = self.root_id {
            self.remove(last_root_id);
            self.root_id = None;
        }
        let data = data.into();
        self.slab = data.slab;
        self.root_id = data.root_id;
    }

    pub(crate) fn get(&self, node_id: usize) -> Option<&SlabTreeEntry<T>> {
        self.slab.get(node_id)
    }

    pub(crate) fn get_mut(&mut self, node_id: usize) -> Option<&mut SlabTreeEntry<T>> {
        self.slab.get_mut(node_id)
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
        if let Some(entry) = self.slab.try_remove(node_id) {
            self.record_data_change();

            // remove fro parents's child list
            if let Some(parent_id) = entry.parent_id {
                let parent = self.get_mut(parent_id).unwrap();
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

    fn sort_children(&mut self, children: &mut [usize], sorter: &SorterFn<T>) {
        children.sort_by(|child_id_a, child_id_b| {
            let entry_a = self.get(*child_id_a).unwrap();
            let entry_b = self.get(*child_id_b).unwrap();
            sorter.cmp(&entry_a.record, &entry_b.record)
        });

        for child_id in children {
            self.sort_node(*child_id, sorter);
        }
    }

    pub(crate) fn sort_node(&mut self, node_id: usize, sorter: &SorterFn<T>) {
        self.record_data_change();
        let mut children = self.get_mut(node_id).unwrap().children.take();
        if let Some(children) = &mut children {
            self.sort_children(children, sorter);
        }
        self.get_mut(node_id).unwrap().children = children;
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
            self.get(parent_id).unwrap().level + 1
        } else {
            0
        };

        let vacant_entry = self.slab.vacant_entry();
        let node_id =  vacant_entry.key();

        let entry = SlabTreeEntry {
            parent_id,
            level,
            record,
            expanded: false,
            children: None,
        };
        vacant_entry.insert(entry);
        node_id
    }
}
