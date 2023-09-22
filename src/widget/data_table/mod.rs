//! Flexible data table/tree widget.

mod events;
pub use events::{DataTableHeaderKeyboardEvent, DataTableKeyboardEvent, DataTableMouseEvent};

mod header_state;
pub(crate) use header_state::HeaderState;

mod resizable_header;
pub(crate) use resizable_header::ResizableHeader;

mod header_group;
pub(crate) use header_group::{
    create_indexed_header_list, IndexedHeader, IndexedHeaderGroup, IndexedHeaderSingle,
};
pub use header_group::{DataTableHeader, DataTableHeaderGroup};

mod row_render_callback;
pub use row_render_callback::{
    DataTableRowRenderArgs, DataTableRowRenderCallback, IntoOptionalDataTableRowRenderCallback,
};

mod row;
pub(crate) use row::DataTableRow;

mod cell_render_callback;
pub use cell_render_callback::{DataTableCellRenderArgs, DataTableCellRenderer};

mod header_render_callback;
pub use header_render_callback::{
    DataTableHeaderRenderArgs, DataTableHeaderRenderer, DataTableHeaderTableLink,
};

mod column;
pub use column::DataTableColumn;

mod header_widget;
pub(crate) use header_widget::HeaderWidget;

#[allow(clippy::module_inception)]
mod data_table;
pub(crate) use data_table::HeaderMsg;
pub use data_table::{DataTable, PwtDataTable, RowSelectionStatus};

use yew::prelude::*;
use yew::virtual_dom::VList;

use super::Row;
use crate::props::{ContainerBuilder, WidgetBuilder};
use crate::state::TreeStore;

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
///
/// This function is deprecated, please use the normal `render` function of a
/// [DataTableColumn] and provide a TreeStore to the column.
#[deprecated]
pub fn render_tree_node<T: 'static>(
    args: &mut DataTableCellRenderArgs<T>,
    render: impl Fn(&T) -> (Option<String>, String),
) -> Html {
    let (class, text) = render(args.record);
    let class = class.unwrap_or(String::new());

    let content = html! {<><i {class}/>{text}</>};

    render_tree_node_impl(args, content, None)
}

pub(crate) fn render_tree_node_impl<T>(
    args: &mut DataTableCellRenderArgs<T>,
    content: Html,
    tree_store: Option<TreeStore<T>>,
) -> Html {
    let mut list: VList = VList::new();
    for _ in 0..args.level() {
        list.push(html! { <span class="pwt-ps-4"/> });
    }

    let indent: Html = list.into();

    let expander = if args.is_leaf() {
        html! {<i role="none" class="fa fa-fw pwt-pe-1"/>}
    } else {
        let caret = match args.is_expanded() {
            true => "pwt-tree-expander fa fa-fw fa-caret-down pwt-pe-1",
            false => "pwt-tree-expander fa fa-fw fa-caret-right pwt-pe-1",
        };

        let onclick = {
            let key = args.record_key.clone();
            let tree_store = tree_store;
            move |_| {
                if let Some(store) = &tree_store {
                    if let Some(mut node) = store.write().lookup_node_mut(&key) {
                        node.set_expanded(!node.expanded());
                    }
                }
            }
        };
        html! {<i role="none" class={caret} {onclick}/>}
    };
    Row::new()
        .class(crate::css::AlignItems::Baseline)
        .with_child(indent)
        .with_child(expander)
        .with_child(content)
        .into()
}

/// Column render function generating the row number.
pub fn render_row_number<T>(args: &mut DataTableCellRenderArgs<T>) -> Html {
    html!{format!("{}", args.row_index())}
}

/// Column render function generating an selection indicator (checkbox).
pub fn render_selection_indicator<T>(args: &mut DataTableCellRenderArgs<T>) -> Html {
    let class = classes!(
        "pwt-no-outline",
        "fa",
        "fa-fw",
        if args.is_selected() { "fa-check-square-o" } else { "fa-square-o" }
    );

    let aria_checked = if args.is_selected() { "true" } else { "false" };

    let onclick = Callback::from({
        let selection = args.selection();
        let record_key = args.record_key.clone();
        move |_| {
            if let Some(selection) = &selection {
                selection.toggle(record_key.clone());
            }
        }
    });

    html!{
        <i {class} {onclick} role="checkbox" aria-checked={aria_checked} aria-label="select"/>
    }
}

/// Header rendering function generating a selection checkbox (select
/// all or none).
pub fn render_selection_header<T>(args: &mut DataTableHeaderRenderArgs<T>) -> Html {
    let status = args.selection_status();
    let class = classes!(
        "pwt-no-outline",
        "fa",
        "fa-fw",
        match status {
            RowSelectionStatus::Nothing => "fa-square-o",
            RowSelectionStatus::Some => "fa-plus-square-o",
            RowSelectionStatus::All => "fa-check-square-o",
        },
    );

    let aria_checked = match status {
        RowSelectionStatus::Nothing => "false",
        RowSelectionStatus::Some => "mixed",
        RowSelectionStatus::All => "true",
    };

    let link = args.link();
    let onclick = Callback::from(move |_| {
        link.send_toggle_select_all();
    });

    html!{
        <i {class} {onclick} role="checkbox" aria-checked={aria_checked} aria-label="select all"/>
    }
}
