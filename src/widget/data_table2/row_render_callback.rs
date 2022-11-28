use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::html::IntoPropValue;

use crate::state::DataNode;

/// Row render callback arguments.
///
/// This can be used to set additional CSS classes and attributes on
/// the table row.
pub struct DataTableRowRenderArgs<'a, T> {
    // The data node.
    pub(crate) node: &'a dyn DataNode<T>,
    // Row index.
    pub(crate) row_index: usize,

    // Select flag is set when the row is selected.
    pub(crate) selected: bool,

    /// Row class. This attribute may be modified to change the
    /// appearance of the row.
    pub class: Classes,
    /// Additional row attributes (style, ...)
    pub attributes: IndexMap<AttrValue, AttrValue>,
}

impl<'a, T> DataTableRowRenderArgs<'a, T> {

    /// Return the data node.
    pub fn node(&self) -> &dyn DataNode<T> {
        self.node
    }

    /// Returns the row index.
    pub fn row_index(&self) -> usize {
        self.row_index
    }

    /// Returns if the cell is selected.
    pub fn is_selected(&self) -> bool {
        self.selected
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

/// DataTable row render callback.
///
/// This can be used to set additional CSS classes and attributes on
/// the table row.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableRowRenderCallback<T>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableRowRenderArgs<T>)>
);

impl<T> DataTableRowRenderCallback<T> {
    /// Creates a new instance.
    pub fn new(renderer: impl 'static + Fn(&mut DataTableRowRenderArgs<T>)) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, args: &mut DataTableRowRenderArgs<T>) {
        (self.0)(args);
    }
}

impl<T, F: 'static + Fn(&mut DataTableRowRenderArgs<T>)> From<F> for DataTableRowRenderCallback<T> {
    fn from(f: F) -> Self {
        DataTableRowRenderCallback::new(f)
    }
}

pub trait IntoOptionalDataTableRowRenderCallback<T> {
    fn into_optional_row_render_cb(self) -> Option<DataTableRowRenderCallback<T>>;
}

impl<T> IntoOptionalDataTableRowRenderCallback<T> for Option<DataTableRowRenderCallback<T>> {
    fn into_optional_row_render_cb(self) -> Option<DataTableRowRenderCallback<T>> {
        self
    }
}

impl<T, F: 'static + Fn(&mut DataTableRowRenderArgs<T>)> IntoOptionalDataTableRowRenderCallback<T> for F {
    fn into_optional_row_render_cb(self) -> Option<DataTableRowRenderCallback<T>> {
        Some(DataTableRowRenderCallback::new(self))
    }
}
