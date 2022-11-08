use std::cmp::Ordering;

use slab::Slab;

pub(crate) struct SlabTreeEntry<T> {
    pub(crate) parent_id: Option<usize>,
    pub(crate) level: usize,
    pub(crate) record: T,
    pub(crate) expanded: bool,
    pub(crate) children: Option<Vec<usize>>,
}

/// Tree implementation backup by a vector ([Slab])
///
/// # Note
///
/// The API is carefully designed to avoid multiple mutable references
/// to the same node, i.e. there is no interface to get a mutable
/// reference to the parent node. Because of this, it is also impossible to
/// have methods like `node.remove()` (remove self from parent).
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

        /// Number of children
        pub fn children_count(&self) -> usize {
            let entry = match self.tree.get(self.node_id) {
                Some(entry) => entry,
                None => return 0,
            };
            match &entry.children {
                Some(children) => children.len(),
                None => 0,
            }
        }

        pub fn child(&self, pos: usize) -> Option<$R> {
            let entry = match self.tree.get(self.node_id) {
                Some(entry) => entry,
                None => return None,
            };

            let child_id = match &entry.children {
                Some(children) => {
                    match children.get(pos) {
                        Some(child_id) => *child_id,
                        None => return None,
                    }
                }
                None => return None,
            };

            Some(<$R>::new(self.tree, child_id))
        }

        /// Visit all children in pre-order
        pub fn visit_children(&self, visitor: &mut impl FnMut(&$R)) {
            for i in 0..self.children_count() {
                let child = self.child(i).unwrap();
                child.visit(visitor);
            }
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
        pub fn sort(&mut self)
        where
            T: Ord,
        {
            self.tree.sort_node(self.node_id, &mut T::cmp);
        }

        pub fn sort_by<F>(&mut self, mut compare: F)
        where
            F: FnMut(&T, &T) -> Ordering,
        {
            self.tree.sort_node(self.node_id, &mut compare);
        }

        /// Get a mutable ref to the child at position `pos`.
        pub fn child_mut(&mut self, pos: usize) -> Option<$M> {
            let entry = match self.tree.get(self.node_id) {
                Some(entry) => entry,
                None => return None,
            };

            let child_id = match &entry.children {
                Some(children) => {
                    match children.get(pos) {
                        Some(child_id) => *child_id,
                        None => return None,
                    }
                }
                None => return None,
            };

            Some(<$M>::new(self.tree, child_id))
        }

        /// Remove the child at position `pos`.
        pub fn remove_child(&mut self, pos: usize) -> Option<T> {
            let child_id = match self.child(pos) {
                Some(child) => child.node_id,
                None => return None,
            };

            self.tree.remove_node_id(child_id)
        }

        /// Visit a subtree in pre-order (mutable)
        pub fn visit_mut(&mut self, visitor: &mut impl FnMut(&mut $M)) {
            visitor(self);
            self.visit_children_mut(visitor);
        }

        /// Visit all children in pre-order (mutable)
        pub fn visit_children_mut(&mut self, visitor: &mut impl FnMut(&mut $M)) {
            for i in 0..self.children_count() {
                let mut child = self.child_mut(i).unwrap();
                child.visit_mut(visitor);
            }
        }
    }
}

impl<'a, T> SlabTreeNodeRef<'a, T> {
    impl_slab_node_ref!{SlabTreeNodeRef<T>}

    /// Iterate over children.
    pub fn children(&self) -> SlabTreeChildren<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        SlabTreeChildren {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    /// Visit a subtree in pre-order
    pub fn visit(&self, visitor:  &mut impl FnMut(&SlabTreeNodeRef<T>)) {
        visitor(self);
        self.visit_children(visitor);
    }
}

impl<'a, T> SlabTreeNodeMut<'a, T> {
    impl_slab_node_ref!{SlabTreeNodeRef<T>}
    impl_slab_node_mut!{SlabTreeNodeMut<T>}

    /// Iterate over children.
    pub fn children(&self) -> SlabTreeChildren<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        SlabTreeChildren {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    /// Iterate over children (mutable).
    pub fn children_mut(&mut self) -> SlabTreeChildrenMut<T> {
        let entry = self.tree.get(self.node_id).unwrap();
        let pos = entry.children.is_some().then(|| 0);
        SlabTreeChildrenMut {
            node_id: self.node_id,
            tree: self.tree,
            pos,
        }
    }

    pub fn visit(&self, visitor: &mut impl FnMut(&SlabTreeNodeRef<T>)) {
        let node_ref = SlabTreeNodeRef::new(self.tree, self.node_id);
        visitor(&node_ref);
        self.visit_children(visitor);
    }
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
            self.remove_node_id(last_root_id);
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
            self.remove_node_id(last_root_id);
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

    pub(crate) fn remove_node_id(&mut self, node_id: usize) -> Option<T> {
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
                    self.remove_node_id(child_id);
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

    fn sort_children<F>(&mut self, children: &mut [usize], compare: &mut F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        children.sort_by(|child_id_a, child_id_b| {
            let entry_a = self.get(*child_id_a).unwrap();
            let entry_b = self.get(*child_id_b).unwrap();
            compare(&entry_a.record, &entry_b.record)
        });

        for child_id in children {
            self.sort_node(*child_id, compare);
        }
    }

    pub(crate) fn sort_node<F>(&mut self, node_id: usize, compare: &mut F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.record_data_change();
        let mut children = self.get_mut(node_id).unwrap().children.take();
        if let Some(children) = &mut children {
            self.sort_children(children, compare);
        }
        self.get_mut(node_id).unwrap().children = children;
    }

    /// Sort the tree node recursively
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(&mut T::cmp);
    }

    /// Sort the tree recursively
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.record_data_change();
        if let Some(root_id) = self.root_id {
            self.sort_node(root_id, &mut compare);
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

/// [SlabTree] iterator over a node`s children.
pub struct SlabTreeChildren<'a, T> {
    pos: Option<usize>,
    node_id: usize,
    tree: &'a SlabTree<T>,
}

impl<'a, T> Iterator for SlabTreeChildren<'a, T> {
    type Item = SlabTreeNodeRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.pos {
            Some(pos) => pos,
            None => return None,
        };

        let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let child_id = match &entry.children {
            Some(children) => {
                match children.get(pos) {
                    Some(child_id) => *child_id,
                    None => return None,
                }
            }
            None => return None,
        };

        self.pos = Some(pos + 1);

        Some(SlabTreeNodeRef {
            node_id: child_id,
            tree: self.tree,
        })
    }
}

/// [SlabTree] iterator over a node`s children (mutable).
pub struct SlabTreeChildrenMut<'a, T> {
    pos: Option<usize>,
    node_id: usize,
    tree: &'a mut SlabTree<T>,
}

impl<'a, T> Iterator for SlabTreeChildrenMut<'a, T> {
    type Item = SlabTreeNodeMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.pos {
            Some(pos) => pos,
            None => return None,
        };
        let entry = match self.tree.get(self.node_id) {
            Some(entry) => entry,
            None => return None,
        };

        let child_id = match &entry.children {
            Some(children) => {
                match children.get(pos) {
                    Some(child_id) => *child_id,
                    None => return None,
                }
            }
            None => return None,
        };

        self.pos = Some(pos + 1);

        let child = SlabTreeNodeMut {
            node_id: child_id,
            tree: self.tree,
        };

        let child = unsafe {
            std::mem::transmute::<SlabTreeNodeMut<T>, SlabTreeNodeMut<'a, T> >(child)
        };

        Some(child)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn node_to_string(node: &mut SlabTreeNodeMut<usize>) -> String {
        fn print_node(node: &mut SlabTreeNodeMut<usize>, out: &mut String) {
            out.push_str(&format!("{}", node.record()));
            let count = node.children_count();
            if count > 0 {
                out.push('{');
                for i in 0..count {
                    if i > 0 { out.push(','); }
                    print_node(&mut node.child_mut(i).unwrap(), out);
                }
                out.push('}');
            }
        }

        let mut out = String::new();
        out.push('{');
        print_node(node, &mut out);
        out.push('}');
        out
    }

    #[test]
    fn test_basics() {
        let mut tree = SlabTree::new();

        let mut root = tree.set_root(0);
        assert_eq!(node_to_string(&mut root), "{0}");

        root.append(1);
        root.append(3);
        root.append(2);

        assert_eq!(node_to_string(&mut root), "{0{1,3,2}}");

        root.sort();
        assert_eq!(node_to_string(&mut root), "{0{1,2,3}}");

        root.remove_child(1);
        assert_eq!(node_to_string(&mut root), "{0{1,3}}");
        root.remove_child(1);
        assert_eq!(node_to_string(&mut root), "{0{1}}");
        root.remove_child(1);
        assert_eq!(node_to_string(&mut root), "{0{1}}");
        root.remove_child(0);
        assert_eq!(node_to_string(&mut root), "{0}");
    }

    #[test]
    fn test_serde() {
        use serde_json::json;

        let tree_data = json!({
            "record": 0,
            "expanded": true,
            "children": [
                { "record": 1 },
                { "record": 4 },
                {
                    "record": 3,
                    "expanded": true,
                    "children": [
                        { "record": 31 },
                        { "record": 33 },
                        { "record": 32 },
                    ],
                },
            ],
        });

        let mut tree: SlabTree<usize> = serde_json::from_value(tree_data).unwrap();
        assert_eq!(node_to_string(&mut tree.root_mut().unwrap()), "{0{1,4,3{31,33,32}}}");

        let mut node_count = 0;
        let mut max_level = 0;
        tree.root().unwrap().visit(&mut |node| {
            node_count += 1;
            max_level = max_level.max(node.level());
        });
        assert_eq!(node_count, 7);
        assert_eq!(max_level, 2);

        tree.sort();

        let text = serde_json::to_string_pretty(&tree).unwrap();
        let mut tree: SlabTree<usize> = serde_json::from_str(&text).unwrap();
        assert_eq!(node_to_string(&mut tree.root_mut().unwrap()), "{0{1,3{31,32,33},4}}");
    }

}
