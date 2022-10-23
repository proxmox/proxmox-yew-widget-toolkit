use slab::Slab;

use crate::props::SorterFn;

pub(crate) struct SlabTreeEntry<T> {
    //pub(crate) node_id: usize,
    pub(crate) parent_id: Option<usize>,
    pub(crate) level: usize,
    pub(crate) record: T,
    pub(crate) expanded: bool,
    pub(crate) children: Option<Vec<usize>>,
}

pub struct SlabTree<T> {
    children: Option<Vec<usize>>,

    tree: Slab<SlabTreeEntry<T>>,

    pub(crate) version: usize, // for change tracking
    pub(crate) linear_view: Vec<usize>, // node_id list
}

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

    /// Get the expanded flag
    pub fn get_expanded(&self) -> bool {
        self.tree.tree.get(self.node_id).unwrap().expanded
    }

    /// Set the expanded flag
    pub fn set_expanded(&mut self, expanded: bool) {
        if self.get_expanded() != expanded {
            self.tree.record_data_change();
            let entry = self.tree.tree.get_mut(self.node_id).unwrap();
            entry.expanded = expanded;
        }
    }

    /// Mutable reference to the node data.
    pub fn record(&mut self) -> &mut T {
        self.tree.record_data_change();
        let entry = self.tree.tree.get_mut(self.node_id).unwrap();
        &mut entry.record
    }

    /// Returns the unique node id.
    pub fn node_id(&self) -> usize {
        self.node_id
    }

    /// Sort the tree node recursively
    pub fn sort(&mut self, sorter: &SorterFn<T>) {
        self.tree.sort_node(self.node_id, sorter);
    }
}

impl<T> SlabTree<T> {

    pub fn new() -> Self {
        let tree = Slab::new();
        Self {
            tree,
            children: None,
            version: 0,
            linear_view: Vec::new(),
        }
    }

    fn record_data_change(&mut self) {
        self.version += 1;
        // fixme: remove/update linear view
    }

    pub fn append(&mut self, record: T) -> SlabTreeNodeMut<T> {
        let node_id = self.insert_entry(record, None);
        if let Some(children) = &mut self.children {
            children.push(node_id);
        } else {
            self.children = Some(vec![node_id]);
        }
        SlabTreeNodeMut {
            node_id,
            tree: self,
        }
    }

    pub(crate) fn get(&self, node_id: usize) -> Option<&SlabTreeEntry<T>> {
        self.tree.get(node_id)
    }

    pub(crate) fn get_mut(&mut self, node_id: usize) -> Option<&mut SlabTreeEntry<T>> {
        self.tree.get_mut(node_id)
    }

    pub(crate) fn get_node_ref_mut(&mut self, node_id: usize) -> SlabTreeNodeMut<T> {
        SlabTreeNodeMut {
            node_id,
            tree: self,
        }
    }

    pub fn remove(&mut self, node_id: usize) -> Option<T> {
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
            Some(entry.record)
        } else {
            None
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
        if let Some(mut children) = self.children.take() {
            self.sort_children(&mut children, sorter);
            self.children = Some(children);
        }
    }

    fn insert_entry(&mut self, record: T, parent_id: Option<usize>) -> usize  {
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
}
