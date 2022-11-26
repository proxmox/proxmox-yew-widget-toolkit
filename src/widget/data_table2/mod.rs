mod events;
pub use events::{DataTableKeyboardEvent, DataTableMouseEvent};

mod header_state;
pub(crate) use header_state::HeaderState;

mod resizable_header;
pub(crate) use resizable_header::ResizableHeader;

mod header_group;
pub use header_group::{DataTableHeader, DataTableHeaderGroup};
pub(crate) use header_group::{
    create_indexed_header_list, IndexedHeader, IndexedHeaderSingle, IndexedHeaderGroup,
};

mod column;
pub use column::{DataTableColumn, DataTableColumnRenderArgs};

mod header_widget;
pub(crate) use header_widget::HeaderWidget;

mod data_table;
pub use data_table::{DataTable, PwtDataTable};

use yew::prelude::*;
use yew::virtual_dom::VList;

use crate::state::DataNode;

// Note: this could be use to generate more complex layouts with tree lines...
fn get_indent<T>(item: &dyn DataNode<T>, indent: &mut VList) {
    if item.level() == 0 { return; }
    if let Some(parent) = item.parent() {
        get_indent(&*parent, indent);
        let space = html!{ <span class="pwt-ps-4"/> };
        indent.push(space);
    }
}

/// Helper function to render tree nodes.
///
/// This function generates a tree node with:
///
/// - correct indentation
/// - a caret indicator to show if the node is expanded
/// - an optional icon
///
/// The passed render function gets the record as parameter and should
/// return a tuple containing the optional icon class and the node
/// text.
pub fn render_tree_node<T>(
    args: &mut DataTableColumnRenderArgs<T>,
    render: impl Fn(&T) -> (Option<String>, String),
) -> Html {
    let node = args.node();

    let record = node.record();
    let (class, content) = render(&*record);
    let class = class.unwrap_or(String::new());

    let mut list: VList = VList::new();
    get_indent(node, &mut list);
    let indent: Html = list.into();

    let leaf = node.is_leaf();
    if leaf {
        html!{<span class="pwt-user-select-none" role="none">{indent.clone()}<i {class}/>{content}</span>}
    } else {
        let carret = match node.expanded() {
            true => "fa fa-fw fa-caret-down pwt-pe-1",
            false => "fa fa-fw fa-caret-right pwt-pe-1",
        };
        html!{
            <span class="pwt-user-select-none" role="none">
                {indent.clone()}
                <i aria-hidden="true" role="none" class={carret}/>
                <i {class}/>
                {content}
            </span>
        }
    }
}
