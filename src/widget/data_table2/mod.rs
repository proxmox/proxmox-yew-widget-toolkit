mod header_state;
pub(crate) use header_state::HeaderState;

mod resizable_header;
pub use resizable_header::{ResizableHeader, PwtResizableHeader};

mod header_group;
pub use header_group::{Header, HeaderGroup};
pub(crate) use header_group::{
    create_indexed_header_list, IndexedHeader, IndexedHeaderSingle, IndexedHeaderGroup,
};

mod column;
pub use column::DataTableColumn;

mod slab_tree;
pub use slab_tree::{SlabTree, SlabTreeNodeMut};

mod tree_store;
pub use tree_store::*;

mod tree;
pub use tree::*;

mod store;
pub use store::*;

mod tree_filter;
pub use tree_filter::{TreeFilter, DataCollection, DataNode, DataNodeDerefGuard, ExtractPrimaryKey};

//mod header_menu;
//pub(crate) use header_menu::HeaderMenu;

mod data_table_header;
pub use data_table_header::*;

mod data_table;
pub use data_table::*;


// Note: RenderFn<dyn DataNode<T>> does not work (?Sized problems), so
// we define a separate render function.
use derivative::Derivative;
use std::rc::Rc;
use yew::Html;

/// Wraps `Rc` around `Fn` so it can be passed as a prop.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct RenderDataNode<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&dyn DataNode<T>) -> Html>
);

impl<T> RenderDataNode<T> {
    /// Creates a new [`RenderFn`]
    pub fn new(renderer: impl 'static + Fn(&dyn DataNode<T>) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, node: &dyn DataNode<T>) -> Html {
        (self.0)(node)
    }
}

impl<T, F: 'static + Fn(&dyn DataNode<T>) -> Html> From<F> for RenderDataNode<T> {
    fn from(f: F) -> Self {
        RenderDataNode::new(f)
    }
}

