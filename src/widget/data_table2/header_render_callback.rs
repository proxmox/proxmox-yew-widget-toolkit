use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::html::IntoPropValue;

use super::{HeaderMsg, RowSelectionStatus};

/// A context which allows sending messages to the data table..
#[derive(Derivative)]
#[derivative(Clone(bound=""))]
pub struct DataTableHeaderTableLink<T: 'static> {
     pub(crate) on_message: Callback<HeaderMsg<T>>,
}

impl<T: 'static> DataTableHeaderTableLink<T> {
    pub fn send_toggle_select_all(&self) {
        self.on_message.emit(HeaderMsg::ToggleSelectAll);
    }
}

/// Header render function arguments.
pub struct DataTableHeaderRenderArgs<T: 'static> {
    // Column index.
    pub(crate) column_index: usize,
    // Selection status from the table.
    pub(crate) selection_status: RowSelectionStatus,

    pub(crate) link: DataTableHeaderTableLink<T>,

    /// Cell class. This attribute may be modified to change the
    /// appearance of the header cell.
    pub class: Classes,
    /// Additional header cell attributes (style, colspan, ...)
    pub attributes: IndexMap<AttrValue, AttrValue>,
}

impl<T: 'static> DataTableHeaderRenderArgs<T> {

    /// Returns the column index.
    pub fn columns_index(&self) -> usize {
        self.column_index
    }

    /// Row selection status from the table.
    pub fn selection_status(&self) -> RowSelectionStatus {
        self.selection_status
    }

    /// Method to set additional html attributes on the header cell
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

    /// Method to add a CSS class to the header cell
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Returns a context to send commands to the data table.
    pub fn link(&self) -> DataTableHeaderTableLink<T> {
        self.link.clone()
    }
}


/// DataTable cell render callback.
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct DataTableHeaderRenderer<T: 'static>(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&mut DataTableHeaderRenderArgs<T>) -> Html>
);

impl<T: 'static> DataTableHeaderRenderer<T> {
    /// Creates a new instance.
    pub fn new(renderer: impl 'static + Fn(&mut DataTableHeaderRenderArgs<T>) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
    /// Apply the render function
    pub fn apply(&self, args: &mut DataTableHeaderRenderArgs<T>) -> Html {
        (self.0)(args)
    }
}

impl<T, F: 'static + Fn(&mut DataTableHeaderRenderArgs<T>) -> Html> From<F> for DataTableHeaderRenderer<T> {
    fn from(f: F) -> Self {
        DataTableHeaderRenderer::new(f)
    }
}
