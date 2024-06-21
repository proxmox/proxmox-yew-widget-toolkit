use std::marker::PhantomData;
use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{ContainerBuilder, WidgetBuilder, WidgetStyleBuilder};
use crate::state::Selection;
use crate::widget::Container;

use super::{
    CellConfiguration, DataTableCellRenderArgs, DataTableColumn, DataTableRowRenderArgs,
    DataTableRowRenderCallback,
};

/// DataTable row properties.
///
/// We implement a table row a separate Component to minimize VDOM
/// diff size.
#[derive(Derivative, Properties)]
#[derivative(Clone, PartialEq)]
pub(crate) struct DataTableRow<T: Clone + PartialEq + 'static> {
    pub selection: Option<Selection>,
    pub unique_table_id: AttrValue,
    pub record: T,
    pub record_key: Key,
    pub row_num: usize,
    // List of currently visible columns
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    pub columns: Rc<Vec<DataTableColumn<T>>>,
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    pub column_hidden: Rc<Vec<bool>>,
    pub min_row_height: usize,
    pub vertical_align: Option<AttrValue>,
    pub cell_config: Rc<CellConfiguration>,
    pub row_render_callback: Option<DataTableRowRenderCallback<T>>,

    #[prop_or_default]
    pub selected: bool,
    pub active_cell: Option<usize>,
    #[prop_or_default]
    pub has_focus: bool,
    #[prop_or_default]
    pub is_expanded: bool,
    #[prop_or_default]
    pub is_leaf: bool,
    #[prop_or(0)]
    pub level: usize,
}

#[doc(hidden)]
pub(crate) struct PwtDataTableRow<T: Clone + PartialEq + 'static> {
    _phantom: PhantomData<T>,
}

impl<T: Clone + PartialEq + 'static> Component for PwtDataTableRow<T> {
    type Message = ();
    type Properties = DataTableRow<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _phantom: PhantomData::<T>,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let item_id = format!("{}-item-{}", props.unique_table_id, props.record_key);

        let aria_expanded = if props.is_leaf {
            None
        } else {
            if props.is_expanded {
                Some("true")
            } else {
                Some("false")
            }
        };

        let mut row = Container::from_tag("tr")
            .key(props.record_key.clone())
            .attribute("role", "row")
            // aria-rowindex does not work, no firefox support?
            .attribute("aria-rowindex", (props.row_num + 1).to_string())
            .attribute("aria-expanded", aria_expanded)
            .attribute(
                "aria-selected",
                if props.selected { "true" } else { "false" },
            )
            .attribute("id", item_id)
            .class((props.active_cell.is_some() && props.has_focus).then(|| "row-cursor"))
            .class(props.selected.then(|| "selected")); // fixme: remove

        if let Some(row_render_callback) = &props.row_render_callback {
            let mut args = DataTableRowRenderArgs {
                record: &props.record,
                record_key: &props.record_key,
                row_index: props.row_num,
                selected: props.selected,
                class: Classes::new(),
                attributes: IndexMap::new(),
            };

            row_render_callback.apply(&mut args);

            if !args.class.is_empty() {
                row.add_class(args.class);
            }

            for (attr_name, attr_value) in args.attributes.into_iter() {
                row.set_attribute(attr_name, attr_value);
            }
        }

        // Make sure our rows have a minimum height
        // Note: setting min-height on <tr> or <td> does not work
        let minheight_cell_style =
            AttrValue::Rc(format!("vertical-align:top;height: {}px;", props.min_row_height).into());

        let mut col_index = 0;
        let mut column_num = 0;

        while let Some(column) = props.columns.get(column_num) {
            if let Some(true) = props.column_hidden.get(column_num) {
                column_num += 1;
                continue;
            }

            let vertical_align = props.vertical_align.to_owned().unwrap_or("baseline".into());
            let text_align = column.justify.to_owned();

            let cell_active = match props.active_cell {
                Some(active_cell) => active_cell == column_num,
                None => false,
            };

            let mut args = DataTableCellRenderArgs {
                selection: props.selection.clone(),
                record: &props.record,
                record_key: &props.record_key,
                row_index: props.row_num,
                column_index: col_index,
                selected: props.selected,
                config: (*props.cell_config).clone(),
                is_expanded: props.is_expanded,
                is_leaf: props.is_leaf,
                level: props.level,
            };

            let cell = column.apply_render(&mut args);

            let mut td = Container::from_tag("td")
                .class(args.config.class)
                .class((cell_active && props.has_focus).then(|| "cell-cursor"))
                .styles(args.config.style)
                .style("vertical-align", vertical_align)
                .style("text-align", text_align)
                .attribute("role", "gridcell")
                .attribute("data-column-num", column_num.to_string())
                .attribute("tabindex", if cell_active { "0" } else { "-1" })
                .with_child(html! {<div role="none">{cell}</div>});

            let mut colspan = 1;

            for (attr_name, attr_value) in args.config.attributes.iter() {
                if attr_name == "colspan" {
                    if let Ok(n) = attr_value.parse::<usize>() {
                        if n > 0 {
                            colspan = n
                        }
                    }
                }
            }

            td.as_std_props_mut().attributes = args.config.attributes;

            col_index += colspan;
            column_num += colspan;

            row.add_child(td);
        }

        row.add_child(html! {<td role="none" style={minheight_cell_style.clone()}/>});
        row.into()
    }
}

impl<T: Clone + PartialEq + 'static> Into<VNode> for DataTableRow<T> {
    fn into(self) -> VNode {
        let key = Some(self.record_key.clone());
        let comp = VComp::new::<PwtDataTableRow<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
