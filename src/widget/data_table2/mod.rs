mod events;
pub use events::{DataTableKeyboardEvent, DataTableMouseEvent};

mod header_state;
pub(crate) use header_state::HeaderState;

mod resizable_header;
pub(crate) use resizable_header::ResizableHeader;

mod header_group;
pub use header_group::{DataTableHeader, DataTableHeaderGroup};
pub(crate) use header_group::{
    create_indexed_header_list, IndexedHeader, IndexedHeaderSingle,
    IndexedHeaderGroup,
};

mod row_render_callback;
pub use row_render_callback::{
    DataTableRowRenderCallback, DataTableRowRenderArgs,
    IntoOptionalDataTableRowRenderCallback,
};

mod cell_render_callback;
pub use cell_render_callback::{DataTableCellRenderer, DataTableCellRenderArgs};

mod column;
pub use column::DataTableColumn;

mod header_widget;
pub(crate) use header_widget::HeaderWidget;

mod data_table;
pub use data_table::{DataTable, PwtDataTable};
pub(crate) use data_table::HeaderMsg;

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
    args: &mut DataTableCellRenderArgs<T>,
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
        html!{<span role="none">{indent.clone()}<i {class}/>{content}</span>}
    } else {
        let carret = match node.expanded() {
            true => "fa fa-fw fa-caret-down pwt-pe-1",
            false => "fa fa-fw fa-caret-right pwt-pe-1",
        };
        html!{
            <span role="none">
                {indent.clone()}
                <i aria-hidden="true" role="none" class={carret}/>
                <i {class}/>
                {content}
            </span>
        }
    }
}

/// Column render function generating the row number.
pub fn render_row_number<T>(args: &mut DataTableCellRenderArgs<T>) -> Html {
    html!{format!("{}", args.row_index())}
}

/// Column render function generating an selection indicator (checkbox).
pub fn render_selection_indicator<T>(args: &mut DataTableCellRenderArgs<T>) -> Html {
    let class = classes!(
        "fa",
        "fa-fw",
        if args.is_selected() { "fa-check-circle-o" } else { "fa-circle-o" }
    );

    let aria_checked = if args.is_selected() { "true" } else { "false" };
    let aria_label = if args.is_selected() { "checked" } else { "not checked" };

    html!{
        <i {class} role="checkbox" aria-checked={aria_checked} aria-label={aria_label}/>
    }
}
