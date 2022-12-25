//! Flexible data table/tree widget.

mod events;
pub use events::{DataTableKeyboardEvent, DataTableHeaderKeyboardEvent, DataTableMouseEvent};

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

mod row;
pub(crate) use row::DataTableRow;

mod cell_render_callback;
pub use cell_render_callback::{DataTableCellRenderer, DataTableCellRenderArgs};

mod header_render_callback;
pub use header_render_callback::{DataTableHeaderRenderer, DataTableHeaderRenderArgs, DataTableHeaderTableLink};

mod column;
pub use column::DataTableColumn;

mod header_widget;
pub(crate) use header_widget::HeaderWidget;

mod data_table;
pub use data_table::{DataTable, PwtDataTable, RowSelectionStatus};
pub(crate) use data_table::HeaderMsg;

use yew::prelude::*;
use yew::virtual_dom::VList;

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
    let record = args.record;
    let (class, content) = render(record);
    let class = class.unwrap_or(String::new());

    let mut list: VList = VList::new();
    for _ in 0..args.level() {
        list.push(html!{ <span class="pwt-ps-4"/> });
    }

    let indent: Html = list.into();

    let leaf = args.is_leaf();
    if leaf {
        html!{<span role="none">{indent.clone()}<i {class}/>{content}</span>}
    } else {
        let carret = match args.is_expanded() {
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
