use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::html::IntoPropValue;

use crate::state::{DataNode, Selection2};

/// Cell render function arguments.
///
/// This can be used to set additional CSS classes and attributes on
/// the table cell.
pub struct DataTableCellRenderArgs<'a, T> {
    // The data node.
    pub(crate) node: &'a dyn DataNode<T>,
    // Row index.
    pub(crate) row_index: usize,
    // Column index.
    pub(crate) column_index: usize,
    // Select flag is set when the cell is selected.
    pub(crate) selected: bool,

    pub(crate) unique_id: AttrValue,
    pub(crate) selection: Option<Selection2>,


    /// Cell class. This attribute may be modified to change the
    /// appearance of the cell.
    pub class: Classes,
    /// Additional cell attributes (style, colspan, ...)
    pub attributes: IndexMap<AttrValue, AttrValue>,
}

impl<'a, T> DataTableCellRenderArgs<'a, T> {

    /// Return the data node.
    pub fn node(&self) -> &dyn DataNode<T> {
        self.node
    }

    /// Returns the row index.
    pub fn row_index(&self) -> usize {
        self.row_index
    }

    /// Returns the column index.
    pub fn columns_index(&self) -> usize {
        self.column_index
    }

    /// Returns the unique table Id.
    ///
    /// This is useful to lookup information in the DOM, i.e. if you
    /// want to find the record associated with a mouse event with
    /// [super::dom_find_record_num]
    pub fn unique_id(&self) -> &str {
        &self.unique_id
    }

    /// Returns if the cell is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Returns the selction model used by the table.
    pub fn selection(&self) -> Option<Selection2> {
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
        if let Some(value) = value.into_prop_value() {
            self.attributes.insert(key.into(), value);
        } else {
            self.attributes.remove(&key.into());
        }
    }

    /// Method to add a CSS class to the table cell
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }
}

/// DataTable cell render callback.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableCellRenderer<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableCellRenderArgs<T>) -> Html>
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

impl<T, F: 'static + Fn(&mut DataTableCellRenderArgs<T>) -> Html> From<F> for DataTableCellRenderer<T> {
    fn from(f: F) -> Self {
        DataTableCellRenderer::new(f)
    }
}
