use std::rc::Rc;

use derivative::Derivative;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::Key;

use crate::props::WidgetStyleBuilder;
use crate::state::Selection;
use crate::widget::data_table::CellConfiguration;

/// Cell render function arguments.
///
/// This can be used to set additional CSS classes and attributes on
/// the table cell.
pub struct DataTableCellRenderArgs<'a, T> {
    // The data node.
    pub(crate) record: &'a T,
    pub(crate) record_key: &'a Key,
    // Row index.
    pub(crate) row_index: usize,
    // Column index.
    pub(crate) column_index: usize,
    // Select flag is set when the cell is selected.
    pub(crate) selected: bool,

    pub(crate) selection: Option<Selection>,

    pub(crate) is_expanded: bool,
    pub(crate) is_leaf: bool,
    pub(crate) level: usize,

    /// Cell Configuration. This attribute may be modified to change the
    /// classes, style and attributes of the cell.
    pub config: CellConfiguration,
}

impl<'a, T> DataTableCellRenderArgs<'a, T> {
    /// Return the data node.
    pub fn record(&self) -> &T {
        self.record
    }

    pub fn key(&self) -> &Key {
        self.record_key
    }

    /// Returns the row index.
    pub fn row_index(&self) -> usize {
        self.row_index
    }

    /// Returns the column index.
    pub fn columns_index(&self) -> usize {
        self.column_index
    }

    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }

    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    pub fn level(&self) -> usize {
        self.level
    }

    /// Returns if the cell is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection> {
        self.selection.clone()
    }

    /// Method to set additional html attributes on the table cell
    ///
    /// Value 'None' removes the attribute.
    pub fn set_attribute(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        self.config.set_attribute(key, value)
    }

    /// Method to add a CSS class to the table cell
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.config.class.push(class);
    }

    /// Method to set additional CSS styles on the table cell
    ///
    /// Value 'None' removes the attribute.
    pub fn set_style(
        &mut self,
        key: impl Into<AttrValue>,
        value: impl IntoPropValue<Option<AttrValue>>,
    ) {
        self.config.set_style(key, value)
    }
}

/// DataTable cell render callback.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct DataTableCellRenderer<T>(
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableCellRenderArgs<T>) -> Html>,
);

impl<T> DataTableCellRenderer<T> {
    /// Creates a new instance.
    pub fn new(renderer: impl 'static + Fn(&mut DataTableCellRenderArgs<T>) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, args: &mut DataTableCellRenderArgs<T>) -> Html {
        (self.0)(args)
    }
}

impl<T, F: 'static + Fn(&mut DataTableCellRenderArgs<T>) -> Html> From<F>
    for DataTableCellRenderer<T>
{
    fn from(f: F) -> Self {
        DataTableCellRenderer::new(f)
    }
}
