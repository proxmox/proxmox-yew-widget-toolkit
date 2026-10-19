use yew::Html;

use crate::widget::data_table::{CellConfiguration, DataTableCellRenderArgs, DataTableColumn};

/// Contains the rendered cell contents plus meta info to properly show it in the data table
pub(crate) struct RenderedCell<'a, T: 'static> {
    pub column_num: usize,
    pub col_index: usize,
    pub colspan: usize,
    pub config: CellConfiguration,
    pub content: Html,
    pub column: &'a DataTableColumn<T>,
}

/// Calculates the necessary index and colspan properties and renders the cell for a row.
pub(crate) fn render_row_cells<'a, T>(
    columns: &'a [DataTableColumn<T>],
    columns_hidden: &[bool],
    mut make_args: impl FnMut(usize) -> DataTableCellRenderArgs<'a, T>,
) -> Vec<RenderedCell<'a, T>> {
    let mut res = Vec::with_capacity(columns.len());

    let mut col_index = 0;
    let mut column_num = 0;

    while let Some(column) = columns.get(column_num) {
        if let Some(true) = columns_hidden.get(column_num) {
            column_num += 1;
            continue;
        }

        let mut args = make_args(col_index);

        let content = column.apply_render(&mut args);

        let mut colspan = 1;

        for (attr_name, attr_value) in args.config.attributes.iter() {
            if attr_name != "colspan" {
                continue;
            }

            match attr_value.parse::<usize>() {
                Ok(n) if n > 0 => colspan = n,
                _ => {}
            }
        }

        res.push(RenderedCell {
            column_num,
            col_index,
            colspan,
            config: args.config,
            content,
            column,
        });

        col_index += colspan;
        column_num += colspan;
    }

    res
}
