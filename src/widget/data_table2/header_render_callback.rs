use std::rc::Rc;

use derivative::Derivative;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::html::IntoPropValue;

use super::HeaderMsg;

/// Header render function arguments.
pub struct DataTableHeaderRenderArgs<T: 'static> {
    // Column index.
    pub(crate) column_index: usize,
    // Select flag is set when the cell is selected.
    pub(crate) all_selected: bool,

    pub on_message: Callback<HeaderMsg<T>>,

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

    /// Returns true if the all cell are selected.
    pub fn all_selected(&self) -> bool {
        self.all_selected
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
