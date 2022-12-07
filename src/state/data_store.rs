use std::ops::{Deref, Range};

use yew::virtual_dom::Key;
use yew::Callback;

use crate::props::{IntoFilterFn, IntoSorterFn};

#[doc(hidden)]
pub trait DataNode<T> {
    /// Access the record data.
    fn record(&self) -> DataNodeDerefGuard<T>;
    /// View level. Can be different than TreeNode::level if view_root
    /// is not set (TreeNode::level - 1).
    fn level(&self) -> usize;
    /// Is the node expanded?
    fn expanded(&self) -> bool;
    /// Is this a leaf node?
    fn is_leaf(&self) -> bool;
    /// Is this the root node?
    fn is_root(&self) -> bool;
    /// Returns the parent node.
    fn parent(&self) -> Option<Box<dyn DataNode<T> + '_>>;
    fn key(&self) -> Key;
}

// Hide docs, because this interface is only used by DataTable and
// Selector, usually not relevant for the user.
//
// This is a minimal interface, only containing thing we really use.
// We may provide a more verbose interface in the future, i.e. allow access
// to unfiltered data.
#[doc(hidden)]
pub trait DataStore: Clone + PartialEq {
    /// The record type
    type Record: 'static;
    /// The collection type used by [Self::set_data].
    type Collection;
    /// Observer type return by [Self::add_listener].
    type Observer;

    fn extract_key(&self, record: &Self::Record) -> Key;

    fn set_data(&self, data: Self::Collection);

    fn data_len(&self) -> usize;

    fn is_empty(&self) -> bool { self.data_len() == 0}

    /// Method to add a change observer.
    ///
    /// The returned observer owns the listener callback. When dropped, the
    /// listener callback will be removed from the [DataStore].
    fn add_listener(&self, cb: impl Into<Callback<()>>) -> Self::Observer;

    fn set_sorter(&self, sorter: impl IntoSorterFn<Self::Record>);
    fn set_filter(&self, filter: impl IntoFilterFn<Self::Record>);
    fn lookup_filtered_record_key(&self, cursor: usize) -> Option<Key>;
    fn filtered_record_pos(&self, key: &Key) -> Option<usize>;
    fn filtered_data_len(&self) -> usize;

    fn filtered_data<'a>(&'a self) ->
        Box<dyn Iterator<Item=(usize, Box<dyn DataNode<Self::Record> + 'a>)> + 'a>;
    fn filtered_data_range<'a>(&'a self, range: Range<usize>) ->
        Box<dyn Iterator<Item=(usize, Box<dyn DataNode<Self::Record> + 'a>)> + 'a>;
}

pub struct DataNodeDerefGuard<'a, T> {
    pub(crate) guard: Box<(dyn Deref<Target=T> + 'a)>,
}

impl<T> Deref for DataNodeDerefGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.guard
     }
}
