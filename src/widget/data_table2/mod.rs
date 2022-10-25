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

//mod tree_filter;
//pub use tree_filter::{TreeFilter, ExtractPrimaryKey};

//mod header_menu;
//pub(crate) use header_menu::HeaderMenu;

mod data_table_header;
pub use data_table_header::*;

mod data_table;
pub use data_table::*;

use yew::virtual_dom::Key;
pub trait ExtractPrimaryKey {
    fn extract_key(&self) -> Key;
}
