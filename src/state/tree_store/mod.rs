mod slab_tree;
pub use slab_tree::{SlabTree, SlabTreeNodeMut};

use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut, Range};

use derivative::Derivative;

use yew::virtual_dom::Key;

//use crate::props::{ExtractKeyFn, SorterFn};
use crate::props::{ExtractKeyFn, ExtractPrimaryKey, IntoSorterFn, IntoFilterFn};
use crate::state::{DataCollection, DataNode, DataNodeDerefGuard};

/// Shared tree store.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct TreeStore<T: 'static> {
    #[derivative(PartialEq(compare_with="inner_state_equal::<T>"))]
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
            inner: Rc::new(RefCell::new(SlabTree::new(extract_key))),
        }
    }
}

impl<T: 'static> TreeStore<T> {

    /// Creates a new instance with the specifies extract key function.
    pub fn with_extract_key(extract_key: impl Into<ExtractKeyFn<T>>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SlabTree::new(extract_key.into()))),
        }
    }

    pub fn read(&self) -> TreeStoreReadGuard<T> {
        TreeStoreReadGuard {
            tree: self.inner.borrow(),
        }
    }

    pub fn write(&self) -> TreeStoreWriteGuard<T> {
        TreeStoreWriteGuard {
            tree: self.inner.borrow_mut(),
        }
    }

    pub fn append(&mut self, record: T) -> SlabTreeNodeShared<T> {
        let mut tree = self.inner.borrow_mut();
        let node = tree.append(record);
        SlabTreeNodeShared {
            node_id: node.node_id(),
            inner: self.inner.clone(),
        }
    }

}

fn inner_state_equal<T>(
    me: &Rc<RefCell<SlabTree<T>>>,
    other: &Rc<RefCell<SlabTree<T>>>
) -> bool {
    Rc::ptr_eq(&me, &other) &&
        me.borrow().version == other.borrow().version
}

impl<T> DataCollection<T> for TreeStore<T> {

    fn extract_key(&self, data: &T) -> Key {
        self.inner.borrow().extract_key.apply(data)
    }

    fn set_sorter(&self, sorter: impl IntoSorterFn<T>) {
        self.inner.borrow_mut().set_sorter(sorter);
    }

    fn set_filter(&self, filter: impl IntoFilterFn<T>) {
        self.inner.borrow_mut().set_filter(filter);
    }

    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key> {
        self.inner.borrow().lookup_filtered_record_key(cursor)
    }

    fn filtered_record_pos(&self, key: &Key) -> Option<usize> {
        self.inner.borrow().filtered_record_pos(key)
    }

    fn filtered_data_len(&self) -> usize {
        self.inner.borrow().filtered_data_len()
    }

    fn filtered_data<'a>(&'a self) -> Box<dyn Iterator<Item=(usize, Box<dyn DataNode<T> + 'a>)> + 'a> {
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
        self.inner.borrow_mut().set_cursor(cursor)
    }
}



pub struct TreeStoreWriteGuard<'a, T> {
    tree: RefMut<'a, SlabTree<T>>,
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
        let child = tree.append(record);

        SlabTreeNodeShared {
            node_id: child.node_id(),
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
